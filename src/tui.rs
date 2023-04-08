use std::thread;
use std::time::Duration;
use pancurses::{endwin, Input, Window};
use crate::guesses::{GuessChar, GuessGrid, GuessStr};
use crate::knowledge::CharKnowledge;
use crate::window_helper::{Color, init, TextScroll, WindowState};

#[derive(PartialEq)]
pub enum UserAction {
    SubmittedRow,
    ChangedKnowledge,
    Other,
}

pub struct MainWindow<const N: usize, const R: usize> {
    window: Window
}

impl<const N: usize, const R: usize> Drop for MainWindow<N, R> {
    fn drop(&mut self) {
        endwin();
    }
}

impl<const N: usize, const R: usize> MainWindow<N, R> {
    pub fn init() -> Self {
        MainWindow {
            window: init()
        }
    }

    pub fn refresh(&self) {
        self.window.touch();
        self.window.refresh();
    }

    pub fn get_input(&self) -> Option<Input> {
        self.window.getch()
    }

    pub fn create_text_scroll(&self, lines: Option<i32>, cols: i32, pos_y: i32, pos_x: i32) -> TextScroll {
        TextScroll::new(&self.window, lines.unwrap_or(self.window.get_max_y()), cols, pos_y, pos_x)
    }

    pub fn draw_guess_grid(&self, grid: &GuessGrid<N, R>) {
        for (i, guess_str) in grid.guesses().iter().enumerate() {
            self.window.mv(3 * (i as i32), 3);
            self.draw_guess_str(guess_str);
        }
        self.draw_active_marker(grid)
    }

    pub fn handle_input(&self, grid: &mut GuessGrid<N, R>, input: Input) -> UserAction {
        let guesses = &mut grid.active_guess_mut();
        match input {
            Input::KeyUp => guesses.cycle_guess_knowledge(true),
            Input::KeyDown => guesses.cycle_guess_knowledge(false),
            Input::KeyRight | Input::Character('\t') => guesses.move_active(true),
            Input::KeyLeft => guesses.move_active(false),
            Input::Character('\x7F') => guesses.unset_ch(),
            Input::Character('\n') => {
                self.handle_newline(grid);
                return UserAction::SubmittedRow;
            }
            Input::Character(c) => guesses.set_ch(c),
            _ => {}
        }
        UserAction::ChangedKnowledge
    }

    fn handle_newline(&self, grid: &mut GuessGrid<N, R>, ) {
        let active_row = grid.active_guess();
        if active_row
            .guesses()
            .iter()
            .any(|c| c.knowledge == CharKnowledge::Unknown)
        {
            self.report_error(grid);
        } else {
            if grid.active_row() + 1 >= grid.guesses().len() {
                self.report_error(grid);
            } else {
                let window_state = WindowState::new(&self.window);
                // deactivate all the char boxes on the current row (it's about to become inactive)
                grid.set_active_char_on_active_row(None);
                // Hide the current active marker
                window_state.set_color(Color::Hidden);
                self.draw_active_marker(grid);

                // Paint the new active marker
                grid.increment_active();
                window_state.set_color(Color::StandardForeground);
                self.draw_active_marker(grid);

                // Set the active char on the current row to 0.
                grid.set_active_char_on_active_row(Some(0));
            }
        }
    }

    fn report_error(&self, grid: &GuessGrid<N, R>) {
        let window_state = WindowState::new(&self.window);
        for color in [Color::Error, Color::StandardForeground].repeat(2) {
            window_state.set_color(color);
            self.draw_active_marker(grid);
            self.window.refresh();
            thread::sleep(Duration::from_millis(80));
        }
    }

    fn draw_active_marker(&self, grid: &GuessGrid<N, R>) {
        self.window.mv(3 * (grid.active_row() as i32) + 1, 1);
        self.window.addstr("➤");
    }

    pub fn draw_guess_str(&self, guess_str: &GuessStr<N>) {
        let (orig_y, orig_x) = self.window.get_cur_yx();
        for (i, guess) in guess_str.guesses().iter().enumerate() {
            self.window.mv(orig_y, orig_x + (i as i32 * 4));
            let style = if guess_str.active_ch().map(|active| active == i).unwrap_or(false) {
                STYLE_ACTIVE
            } else {
                STYLE_INACTIVE
            };
            self.draw_guess_box(guess, &style);
        }
        self.window.mv(orig_y, orig_x);
    }

    fn draw_guess_box(&self, ch: &GuessChar, style: &BoxStyle) {
        let guessed_char = ch.ch.unwrap_or(' ');
        let window_state = WindowState::new(&self.window);

        window_state.set_color(ch.knowledge.color());
        _ = self.window.printw(style.top);
        _ = self.window.mvprintw(
            window_state.orig_y + 1,
            window_state.orig_x,
            format!("{}{}{}", style.vert, guessed_char, style.vert),
        );
        _ = self.window.mvprintw(window_state.orig_y + 2, window_state.orig_x, style.bot);
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
