use crate::analyze::char_stats::CharCounts;
use crate::analyze::scored_chars::ScoredChars;
use crate::guess::guesses::{GuessChar, GuessGrid, GuessStr};
use crate::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use crate::ui::text_scroll_pane::TextScroll;
use crate::ui::window_helper::{init, Color, WindowState};
use crate::word_list::WordList;
use pancurses::{endwin, Input, Window};
use std::cmp::min;
use std::thread;
use std::time::Duration;
use strum::EnumCount;

pub struct MainWindow<const N: usize, const R: usize> {
    window: Window,
    active_row: usize,
    active_col: usize,
}

impl<const N: usize, const R: usize> Drop for MainWindow<N, R> {
    fn drop(&mut self) {
        endwin();
    }
}

impl<const N: usize, const R: usize> MainWindow<N, R> {
    pub fn init() -> Self {
        MainWindow {
            window: init(),
            active_row: 0,
            active_col: 0,
        }
    }

    pub fn run_main_loop(&mut self) {
        let mut guess_grid = GuessGrid::<N, R>::new();
        self.draw_guess_grid(&guess_grid);

        let mut words_window = self.create_text_scroll(None, 30, 0, 28);
        let mut scores_window = self.create_text_scroll(None, 30, 0, 64);
        words_window.set_title(Some("Words whatever 1234567890abcdefghijklmnopqrstuvwzyz".to_string()));
        let mut refresh_words_list = true;

        loop {
            if refresh_words_list {
                let known_constraints = KnownWordConstraints::from_grid(&guess_grid);
                // TODO we can keep just one list on the outside, and whittle it time every time the
                // user presses "enter"
                let mut possible_words = WordList::<N>::get_embedded(10000);
                possible_words.filter(&known_constraints);

                let char_counts = CharCounts::new(&possible_words);
                let scores = ScoredChars::new(&possible_words, &char_counts);
                scores_window.set_texts(
                    scores
                        .all_word_scores()
                        .iter()
                        .take(50)
                        .map(|(word, score)| format!("{}: {:.3}", word, score))
                        .collect(),
                );

                words_window.set_texts(
                    possible_words
                        .words()
                        .iter()
                        .map(|wf| wf.word.to_string())
                        .collect(),
                );
                refresh_words_list = false;
            }

            self.draw_guess_grid(&guess_grid);
            self.refresh();

            let Some(input) = self.get_input() else {
                continue;
            };

            match input {
                Input::KeyUp => {
                    self.cycle_guess_knowledge(&mut guess_grid, true);
                    refresh_words_list = true;
                },
                Input::KeyDown => {
                    self.cycle_guess_knowledge(&mut guess_grid, false);
                    refresh_words_list = true;
                },
                Input::KeyRight => self.move_active_ch(true),
                Input::KeyLeft => self.move_active_ch(false),
                Input::Character(input_ch) => match input_ch {
                    '\x03' => break,                      // ctrl-c
                    '\x04' => words_window.scroll_down(), // ctrl-d
                    '\x15' => words_window.scroll_up(),   // ctrl-u
                    '\n' => self.handle_newline(&mut guess_grid),
                    '\x7F' => {
                        // delete
                        self.unset_active_ch(&mut guess_grid.guess_mut(self.active_row));
                        refresh_words_list = true;
                    }
                    _ => {
                        self.set_active_ch(&mut guess_grid.guess_mut(self.active_row), input_ch);
                        refresh_words_list = true;
                    }
                },
                _ => {}
            };
        }
    }

    pub fn refresh(&self) {
        self.window.touch();
        self.window.refresh();
    }

    pub fn get_input(&self) -> Option<Input> {
        self.window.getch()
    }

    pub fn create_text_scroll(
        &self,
        lines: Option<i32>,
        cols: i32,
        pos_y: i32,
        pos_x: i32,
    ) -> TextScroll {
        TextScroll::new(
            &self.window,
            lines.unwrap_or(self.window.get_max_y()),
            cols,
            pos_y,
            pos_x,
        )
    }

    pub fn draw_guess_grid(&self, grid: &GuessGrid<N, R>) {
        for (row_idx, guess_str) in grid.guesses().iter().enumerate() {
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

    fn handle_newline(&mut self, grid: &mut GuessGrid<N, R>) {
        let active_row = &grid.guesses()[self.active_row];
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

    fn report_error(&self) {
        let window_state = WindowState::new(&self.window);
        for color in [Color::Error, Color::StandardForeground].repeat(2) {
            window_state.set_color(color);
            self.draw_active_marker();
            self.window.refresh();
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn draw_active_marker(&self) {
        self.window
            .mvaddstr(3 * (self.active_row as i32) + 1, 1, "➤");
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

    fn move_active_ch(&mut self, right: bool) {
        self.active_col = incr_usize(self.active_col, N, right, WRAP);
    }

    fn cycle_guess_knowledge(&self, grid: &mut GuessGrid<N, R>, up: bool) {
        let guess_str = &mut grid.guess_mut(self.active_row);

        let guess_ch = guess_str.guess_mut(self.active_col);
        let curr_knowledge = guess_ch.knowledge();
        let next_idx = incr_usize(curr_knowledge as usize, CharKnowledge::COUNT, up, WRAP);
        let next =
            CharKnowledge::from_repr(next_idx).expect(&format!("out of range for {}", next_idx));
        guess_ch.set_knowledge(next);
    }

    fn unset_active_ch(&mut self, guess_str: &mut GuessStr<N>) {
        let old = guess_str.guess_mut(self.active_col).set_ch(None);
        if let None = old {
            self.move_active_ch(false);
        }
    }

    fn set_active_ch(&mut self, guess_str: &mut GuessStr<N>, ch: char) {
        guess_str.guess_mut(self.active_col).set_ch(Some(ch));
        self.move_active_ch(true);
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
