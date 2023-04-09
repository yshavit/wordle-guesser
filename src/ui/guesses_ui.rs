use crate::guess::guesses::{GuessChar, GuessGrid};
use crate::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use crate::ui::widget::Widget;
use crate::ui::window_helper::{Color, WindowState};
use pancurses::{Input, Window};
use std::cmp::min;
use std::thread;
use std::time::Duration;
use strum::EnumCount;

pub struct GuessesUI<const N: usize, const R: usize> {
    window: Window,
    grid: GuessGrid<N, R>,
    active_row: usize,
    active_col: usize,
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
        };
        res.draw_guess_grid();
        res
    }

    pub fn get_known_constraints(&self) -> KnownWordConstraints<N> {
        KnownWordConstraints::from_grid(&self.grid)
    }
}

impl<const N: usize, const R: usize> Widget for GuessesUI<N, R> {
    fn title(&self) -> Option<&str> {
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
            Input::Character(input_ch) => self.set_active_ch(input_ch),
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
        self.draw_active_marker()
    }

    fn draw_guess_box(&self, guess_ch: &GuessChar, style: &BoxStyle) {
        let guessed_char = guess_ch.ch().unwrap_or(' ');
        let window_state = WindowState::new(&self.window);

        window_state.set_color(color_for_knowledge(guess_ch.knowledge()));
        _ = self.window.printw(style.top);
        _ = self.window.mvprintw(
            window_state.orig_y + 1,
            window_state.orig_x,
            format!("{}{}{}", style.vert, guessed_char, style.vert),
        );
        _ = self
            .window
            .mvprintw(window_state.orig_y + 2, window_state.orig_x, style.bot);
    }

    fn draw_active_marker(&self) {
        self.window
            .mvaddstr(3 * (self.active_row as i32) + 1, 1, "➤");
    }

    fn handle_newline(&mut self) {
        let active_row = &self.grid.guesses()[self.active_row];
        if active_row
            .guesses()
            .iter()
            .any(|c| c.knowledge() == CharKnowledge::Unknown)
        {
            self.report_error();
        } else {
            if self.active_row + 1 >= N {
                self.report_error();
            } else {
                let window_state = WindowState::new(&self.window);
                // Hide the current active marker
                window_state.set_color(Color::Hidden);
                self.draw_active_marker();

                // Paint the new active marker
                self.active_row += 1;
                window_state.set_color(Color::StandardForeground);
                self.draw_active_marker();

                // Set the active char on the current row to 0.
                self.active_col = 0;
            }
        }
    }

    fn move_active_ch(&mut self, right: bool) {
        self.active_col = incr_usize(self.active_col, N, right, WRAP);
    }

    fn cycle_guess_knowledge(&mut self, up: bool) {
        let guess_str = self.grid.guess_mut(self.active_row);

        let guess_ch = guess_str.guess_mut(self.active_col);
        let curr_knowledge = guess_ch.knowledge();
        let next_idx = incr_usize(curr_knowledge as usize, CharKnowledge::COUNT, up, WRAP);
        let next =
            CharKnowledge::from_repr(next_idx).expect(&format!("out of range for {}", next_idx));
        guess_ch.set_knowledge(next);
    }

    fn unset_active_ch(&mut self) {
        let guess_str = &mut self.grid.guess_mut(self.active_row);
        let old = guess_str.guess_mut(self.active_col).set_ch(None);
        if let None = old {
            self.move_active_ch(false);
        }
    }

    fn set_active_ch(&mut self, ch: char) {
        let guess_str = &mut self.grid.guess_mut(self.active_row);
        guess_str.guess_mut(self.active_col).set_ch(Some(ch));
        self.move_active_ch(true);
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

fn incr_usize(u: usize, max_exclusive: usize, up: bool, wrap: bool) -> usize {
    match (u.checked_add_signed(if up { 1 } else { -1 }), wrap) {
        (Some(incremented), WRAP) => incremented % max_exclusive,
        (Some(incremented), NO_WRAP) => min(incremented, max_exclusive - 1),
        (None, WRAP) => max_exclusive - 1,
        (None, NO_WRAP) => 0,
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

const WRAP: bool = true;
const NO_WRAP: bool = false;
