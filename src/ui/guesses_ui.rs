use crate::guess::guesses::{GuessChar, GuessGrid};
use crate::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use crate::ui::widget::Widget;
use crate::ui::window_helper::{Color, WindowState};
use crate::util::{incr_usize, WRAP};
use crate::word_list::WordList;
use pancurses::{Input, Window};

use std::cell::Cell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use strum::EnumCount;

pub struct GuessesUI<const N: usize, const R: usize> {
    window: Window,
    grid: GuessGrid<N, R>,
    active_row: usize,
    active_col: usize,
    has_new_knowledge: Cell<bool>,
    possible_words: WordList<N>,
    current_row_inference: [Option<char>; N],
}

impl<const N: usize, const R: usize> GuessesUI<N, R> {
    pub fn new(window: &Window, pos_y: i32, pos_x: i32) -> Self {
        let res = Self {
            window: window
                .subwin((R * 4) as i32, (N * 4 + 2) as i32, pos_y, pos_x)
                .expect("couldn't create entry widget"),
            grid: GuessGrid::new(),
            active_row: 0,
            active_col: 0,
            has_new_knowledge: Cell::new(true),
            possible_words: WordList::std(),
            current_row_inference: [None; N],
        };
        res.draw_guess_grid();
        res
    }

    pub fn handle_new_knowledge<F>(&self, mut handler: F)
    where
        F: FnMut(Rc<WordList<N>>),
    {
        if self.has_new_knowledge.get() {
            let constraints = KnownWordConstraints::from_grid(&self.grid);
            let possible_words = self.possible_words.filter_preview(&constraints);
            handler(Rc::new(possible_words));
            self.has_new_knowledge.set(false);
        }
    }
}

impl<'a, const N: usize, const R: usize> Widget for GuessesUI<N, R> {
    fn title(&self) -> Option<String> {
        None
    }

    fn set_active(&mut self, active: bool) {
        let _old_state = if active {
            let s = WindowState::new(&self.window);
            s.set_color(Color::Hidden);
            Some(s)
        } else {
            None
        };
        self.draw_active_marker();
    }

    fn handle_input(&mut self, input: Input) -> Option<Input> {
        match input {
            Input::KeyUp | Input::KeyDown => self.cycle_guess_knowledge(input == Input::KeyUp),
            Input::KeyRight | Input::KeyLeft => self.move_active_ch(input == Input::KeyRight),
            Input::Character('\n') => self.handle_newline(),
            Input::Character('\x7F') => self.unset_active_ch(), // delete
            Input::Character(input_ch) if input_ch.is_ascii_alphabetic() => {
                if !(self.set_active_ch(input_ch)) {
                    return Some(input);
                } else {
                }
            }
            _ => {
                return Some(input);
            }
        }
        self.draw_guess_grid();
        None
    }
}

impl<const N: usize, const R: usize> GuessesUI<N, R> {
    fn draw_guess_grid(&self) {
        for (row_idx, guess_str) in self.grid.guesses().iter().enumerate() {
            self.window.mv(3 * (row_idx as i32), 3);
            let (orig_y, orig_x) = self.window.get_cur_yx();
            for (ch_idx, guess) in guess_str.guesses().iter().enumerate() {
                self.window.mv(orig_y, orig_x + (ch_idx as i32 * 4));
                let style = if self.active_row == row_idx && self.active_col == ch_idx {
                    STYLE_ACTIVE
                } else {
                    STYLE_INACTIVE
                };
                self.draw_guess_box(guess, &style);
            }
            self.window.mv(orig_y, orig_x);
        }
        let _window_state = if self.fully_guessed() {
            let window_state = WindowState::new(&self.window);
            window_state.set_color(Color::Good);
            Some(window_state)
        } else {
            None
        };
        self.draw_active_marker();
    }

    fn draw_guess_box(&self, guess_ch: &GuessChar, style: &BoxStyle) {
        let guessed_char = guess_ch.ch().unwrap_or(' ');
        let window_state = WindowState::new(&self.window);

        window_state.set_color(color_for_knowledge(guess_ch.knowledge()));
        _ = self.window.addstr(style.top);
        _ = self.window.mvaddstr(
            window_state.orig_y + 1,
            window_state.orig_x,
            format!("{}{}{}", style.vert, guessed_char, style.vert),
        );
        _ = self
            .window
            .mvaddstr(window_state.orig_y + 2, window_state.orig_x, style.bot);
    }

    fn draw_active_marker(&self) {
        self.window
            .mvaddstr(3 * (self.active_row as i32) + 1, 1, "➤");
    }

    fn fully_guessed(&self) -> bool {
        self.grid.guesses()[self.active_row]
            .chars()
            .all(|ch| ch.knowledge() == CharKnowledge::Correct)
    }

    fn handle_newline(&mut self) {
        let active_row = &self.grid.guesses()[self.active_row];
        if active_row
            .guesses()
            .iter()
            .any(|c| c.knowledge() == CharKnowledge::Unknown)
        {
            self.report_error();
        } else if self.active_row + 1 > N {
            self.report_error();
        } else if !self.fully_guessed() {
            let window_state = WindowState::new(&self.window);
            // Hide the current active marker
            window_state.set_color(Color::Hidden);
            self.draw_active_marker();

            // Paint the new active marker
            self.active_row += 1;
            window_state.set_color(Color::StandardForeground);
            self.draw_active_marker();

            // Reify the possible words
            self.possible_words
                .filter(&KnownWordConstraints::from_grid(&self.grid));

            // Get the current inference, and enter it in
            self.current_row_inference = self.grid.known_chars();
            let active_row = self.grid.guess_mut(self.active_row);
            for (idx, inferred) in self.current_row_inference.iter().enumerate() {
                if let Some(ch) = inferred {
                    let cell = active_row.guess_mut(idx);
                    cell.set_ch(*ch);
                    cell.set_knowledge(CharKnowledge::Correct);
                }
            }

            // Set the active char on the current row to the first unknown char
            self.active_col = active_row
                .chars()
                .enumerate()
                .find(|e| e.1.knowledge() != CharKnowledge::Correct)
                .map(|e| e.0)
                .unwrap_or(0);
        }
    }

    fn move_active_ch(&mut self, right: bool) {
        incr_usize(&mut self.active_col, N, right, WRAP);
    }

    fn cycle_guess_knowledge(&mut self, up: bool) {
        let guess_str = self.grid.guess_mut(self.active_row);

        let guess_ch = guess_str.guess_mut(self.active_col);
        let curr_knowledge = guess_ch.knowledge();
        let mut next_idx = curr_knowledge as usize;
        incr_usize(&mut next_idx, CharKnowledge::COUNT, up, WRAP);
        let next =
            CharKnowledge::from_repr(next_idx).expect(&format!("out of range for {}", next_idx));
        guess_ch.set_knowledge(next);
        self.has_new_knowledge.set(true);
    }

    fn unset_active_ch(&mut self) {
        let guess_str = &mut self.grid.guess_mut(self.active_row);
        let old = guess_str.guess_mut(self.active_col).unset_ch();
        match old {
            Some(_) => self.has_new_knowledge.set(true),
            None => self.move_active_ch(false),
        }
    }

    fn set_active_ch(&mut self, ch: char) -> bool {
        let guess_str = &mut self.grid.guess_mut(self.active_row);
        let had_knowledge_before =
            guess_str.guesses()[self.active_col].knowledge() != CharKnowledge::Unknown;
        if guess_str.guess_mut(self.active_col).set_ch(ch) {
            let inferred = &self.current_row_inference[self.active_col];
            if inferred == &guess_str.guesses()[self.active_col].ch() {
                guess_str
                    .guess_mut(self.active_col)
                    .set_knowledge(CharKnowledge::Correct);
            }
            self.move_active_ch(true);
            if had_knowledge_before {
                // We don't have knowledge after set_ch, so if we used to, that's a change.
                self.has_new_knowledge.set(true);
            }
            true
        } else {
            false
        }
    }

    fn report_error(&self) {
        let window_state = WindowState::new(&self.window);
        for color in [Color::Error, Color::StandardForeground].repeat(2) {
            window_state.set_color(color);
            self.draw_active_marker();
            self.window.refresh();
            thread::sleep(Duration::from_millis(80));
        }
    }
}

fn color_for_knowledge(knowledge: CharKnowledge) -> Color {
    match knowledge {
        CharKnowledge::Unknown => Color::StandardForeground,
        CharKnowledge::WrongPosition => Color::Warning,
        CharKnowledge::Correct => Color::Good,
        CharKnowledge::Missing => Color::Error,
    }
}

struct BoxStyle<'a> {
    top: &'a str,
    vert: char,
    bot: &'a str,
}

const STYLE_ACTIVE: BoxStyle = BoxStyle {
    top: "╔═╗",
    vert: '║',
    bot: "╚═╝",
};

const STYLE_INACTIVE: BoxStyle = BoxStyle {
    top: "╭─╮",
    vert: '│',
    bot: "╰─╯",
};
