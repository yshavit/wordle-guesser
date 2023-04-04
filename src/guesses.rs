use std::cmp::min;

use pancurses::Window;
use strum::{EnumCount, FromRepr};

pub struct BoxStyle<'a> {
    top: &'a str,
    vert: char,
    bot: &'a str,
}

const STYLE_ACTIVE: BoxStyle = BoxStyle {
    top: "┏━┓",
    vert: '┃',
    bot: "┗━┛",
};

const STYLE_INACTIVE: BoxStyle = BoxStyle {
    top: "╭─╮",
    vert: '│',
    bot: "╰─╯",
};

#[derive(Copy, Clone, EnumCount, FromRepr)]
pub enum GuessKnowledge {
    Unknown,
    WrongPosition,
    Correct,
    Missing,
}

impl GuessKnowledge {
    pub fn as_i16(&self) -> i16 {
        (*self as usize) as i16
    }
}

pub struct GuessChar {
    knowledge: GuessKnowledge,
    ch: Option<char>,
}

impl GuessChar {
    pub fn draw(&self, window: &Window, style: &BoxStyle) {
        let (orig_y, orig_x) = window.get_cur_yx();
        let ch = self.ch.unwrap_or(' ');

        let (orig_attrs, orig_color) = window.attrget();
        window.color_set(self.knowledge.as_i16());

        _ = window.printw(style.top);
        _ = window.mvprintw(
            orig_y + 1,
            orig_x,
            format!("{}{}{}", style.vert, ch, style.vert),
        );
        _ = window.mvprintw(orig_y + 2, orig_x, style.bot);

        window.attrset(orig_attrs);
        window.color_set(orig_color);
    }

    pub fn set_knowledge(&mut self, knowledge: GuessKnowledge) {
        if let Some(_) = self.ch {
            self.knowledge = knowledge;
        }
    }

    pub fn set_ch(&mut self, ch: Option<char>) {
        self.ch = ch;
        if let None = ch {
            self.knowledge = GuessKnowledge::Unknown;
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
            active: if size > 0 { Some(0) } else { None },
        };
        for _ in 0..size {
            result.guesses.push(GuessChar {
                knowledge: GuessKnowledge::Unknown,
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
                self.active = Some(incr_usize(curr, self.guesses.len(), right, NO_WRAP));
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

pub fn incr_usize(u: usize, max_exclusive: usize, up: bool, wrap: bool) -> usize {
    match (u.checked_add_signed(if up { 1 } else { -1 }), wrap) {
        (Some(incremented), WRAP) => incremented % max_exclusive,
        (Some(incremented), NO_WRAP) => min(incremented, max_exclusive - 1),
        (None, WRAP) =>max_exclusive - 1,
        (None, NO_WRAP) => 0,
    }
}

const WRAP: bool = true;
const NO_WRAP: bool = false;
