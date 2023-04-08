use crate::guesses::{GuessChar, GuessGrid, GuessStr};
use crate::knowledge::CharKnowledge;
use crate::window_helper::{init, Color, TextScroll, WindowState};
use pancurses::{endwin, Input, Window};
use std::cmp::min;
use std::thread;
use std::time::Duration;
use strum::EnumCount;

#[derive(PartialEq)]
pub enum UserAction {
    SubmittedRow,
    ChangedKnowledge,
    Other,
}

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

    pub fn handle_input(&mut self, grid: &mut GuessGrid<N, R>, input: Input) -> UserAction {
        let guesses = &mut grid.guess_mut(self.active_row);
        match input {
            Input::KeyUp => self.cycle_guess_knowledge(guesses, true),
            Input::KeyDown => self.cycle_guess_knowledge(guesses, false),
            Input::KeyRight | Input::Character('\t') => self.move_active_ch(true),
            Input::KeyLeft => self.move_active_ch(false),
            Input::Character('\n') => {
                self.handle_newline(grid);
                return UserAction::SubmittedRow;
            }
            Input::Character('\x7F') => {
                // delete
                self.unset_active_ch(guesses);
            }
            Input::Character(c) => {
                self.set_active_ch(guesses, c);
            }
            _ => {}
        }
        UserAction::ChangedKnowledge
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

        window_state.set_color(guess_ch.knowledge().color());
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

    fn cycle_guess_knowledge(&self, guess_str: &mut GuessStr<N>, up: bool) {
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
