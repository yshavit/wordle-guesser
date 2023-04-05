use std::cmp::min;
use std::thread;
use std::time::Duration;

use pancurses::{Input, Window};
use strum::{EnumCount, FromRepr};
use crate::guesses_ui::GuessKnowledge::{Missing, Unknown};
use crate::window_helper::{Color, WindowState};

pub struct BoxStyle<'a> {
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

#[derive(Copy, Clone, PartialEq, EnumCount, FromRepr)]
pub enum GuessKnowledge {
    Unknown,
    WrongPosition,
    Correct,
    Missing,
}

impl GuessKnowledge {
    fn color(&self) -> Color {
        match self {
            Unknown => Color::StandardForeground,
            GuessKnowledge::WrongPosition => Color::Warning,
            GuessKnowledge::Correct => Color::Good,
            Missing => Color::Error,
        }
    }
}

pub struct GuessChar {
    knowledge: GuessKnowledge,
    ch: Option<char>,
}

impl GuessChar {
    pub fn draw(&self, window: &Window, style: &BoxStyle) {
        let guessed_char = self.ch.unwrap_or(' ');
        let window_state = WindowState::new(window);

        window_state.set_color(self.knowledge.color());
        _ = window.printw(style.top);
        _ = window.mvprintw(
            window_state.orig_y + 1,
            window_state.orig_x,
            format!("{}{}{}", style.vert, guessed_char, style.vert),
        );
        _ = window.mvprintw(window_state.orig_y + 2, window_state.orig_x, style.bot);
    }

    pub fn set_knowledge(&mut self, knowledge: GuessKnowledge) {
        if let Some(_) = self.ch {
            self.knowledge = knowledge;
        }
    }

    pub fn set_ch(&mut self, ch: Option<char>) {
        self.ch = ch;
        if let None = ch {
            self.knowledge = Unknown;
        }
    }
}

pub struct GuessStr {
    guesses: Vec<GuessChar>,
    active: Option<usize>,
}

impl GuessStr {
    pub fn new(size: usize) -> Self {
        let mut result = GuessStr {
            guesses: Vec::with_capacity(size),
            active: None,
        };
        for _ in 0..size {
            result.guesses.push(GuessChar {
                knowledge: Unknown,
                ch: None,
            })
        }
        return result;
    }

    pub fn draw(&self, window: &Window) {
        let (orig_y, orig_x) = window.get_cur_yx();
        for (i, guess) in self.guesses.iter().enumerate() {
            window.mv(orig_y, orig_x + (i as i32 * 4));
            let style = if self.active.map(|active| active == i).unwrap_or(false) {
                STYLE_ACTIVE
            } else {
                STYLE_INACTIVE
            };
            guess.draw(window, &style);
        }
        window.mv(orig_y, orig_x);
    }

    pub fn cycle_guess_knowledge(&mut self, up: bool) {
        if let Some(active) = self.active {
            let guess = self.guesses.get_mut(active).expect("out of bounds");
            let curr_knowledge = guess.knowledge;
            let next_idx = incr_usize(curr_knowledge as usize, GuessKnowledge::COUNT, up, WRAP);
            let next = GuessKnowledge::from_repr(next_idx)
                .expect(&format!("out of range for {}", next_idx));
            guess.set_knowledge(next);
        }
    }

    pub fn move_active(&mut self, right: bool) {
        if self.guesses.is_empty() {
            return;
        }
        match self.active {
            None => {
                if right {
                    self.active = Some(self.guesses.len() - 1)
                } else {
                    self.active = Some(0)
                }
            }
            Some(curr) => {
                self.active = Some(incr_usize(curr, self.guesses.len(), right, WRAP));
            }
        }
    }

    pub fn set_ch(&mut self, ch: char) {
        self.set_ch_direct(Some(ch.to_ascii_uppercase()));
        self.move_active(true);
    }

    pub fn unset_ch(&mut self) {
        if let Some(idx) = self.active {
            let active = self.guesses.get_mut(idx).expect("out of bounds");
            if let None = active.ch {
                self.move_active(false);
            }
            self.set_ch_direct(None)
        }
    }

    pub fn set_ch_direct(&mut self, ch: Option<char>) {
        if let Some(idx) = self.active {
            let active = self.guesses.get_mut(idx).expect("out of bounds");
            active.set_ch(ch);
        }
    }
}

pub struct GuessGrid {
    guesses: Vec<GuessStr>,
    active: usize,
}

impl GuessGrid {
    pub fn new() -> GuessGrid {
        let mut result = GuessGrid {
            guesses: Vec::with_capacity(6),
            active: 0,
        };
        for _ in 0..6 {
            result.guesses.push(GuessStr::new(5))
        }
        result.guesses[0].active = Some(0);
        return result
    }

    pub fn draw(&self, window: &Window) {
        for (i, guess_str) in self.guesses.iter().enumerate() {
            window.mv(3 * (i as i32), 3);
            guess_str.draw(window);
        }
        self.draw_active_marker(window)
    }

    pub fn handle_input(&mut self, window: &Window, input: Input) {
        let guesses = &mut self.guesses[self.active];
        match input {
            Input::KeyUp => guesses.cycle_guess_knowledge(true),
            Input::KeyDown => guesses.cycle_guess_knowledge(false),
            Input::KeyRight => guesses.move_active(true),
            Input::KeyLeft => guesses.move_active(false),
            Input::Character('\n') => self.handle_newline(window),
            Input::Character('\x7F') => guesses.unset_ch(),
            Input::Character(c) => guesses.set_ch(c),
            _ => {}
        }
    }

    fn handle_newline(&mut self, window: &Window) {
        let active_row = &self.guesses[self.active];
        if active_row.guesses.iter().any(|c| c.knowledge == Unknown) {
            self.report_error(window);
        } else {
            if self.active + 1 >= self.guesses.len() {
                self.report_error(window);
            } else {
                let window_state = WindowState::new(window);
                window_state.set_color(Color::Hidden);
                self.guesses[self.active].active = None;
                self.draw_active_marker(window);
                window_state.set_color(Color::StandardForeground);
                self.active += 1;
                self.draw_active_marker(window);
                self.guesses[self.active].active = Some(0);
            }
        }
    }

    fn draw_active_marker(&self, window: &Window) {
        window.mv(3 * (self.active as i32) + 1, 1);
        window.addstr("➤");
    }

    fn report_error(&self, window: &Window) {
        let window_state = WindowState::new(window);
        for color in [Color::Error, Color::StandardForeground].repeat(2) {
            window_state.set_color(color);
            self.draw_active_marker(window);
            window.refresh();
            thread::sleep(Duration::from_millis(80));
        }
    }
}

fn incr_usize(u: usize, max_exclusive: usize, up: bool, wrap: bool) -> usize {
    match (u.checked_add_signed(if up { 1 } else { -1 }), wrap) {
        (Some(incremented), WRAP) => incremented % max_exclusive,
        (Some(incremented), NO_WRAP) => min(incremented, max_exclusive - 1),
        (None, WRAP) =>max_exclusive - 1,
        (None, NO_WRAP) => 0,
    }
}

const WRAP: bool = true;
const NO_WRAP: bool = false;
