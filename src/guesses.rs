use std::cmp::min;
use std::slice::Iter;

use crate::knowledge::CharKnowledge;
use strum::EnumCount;

pub struct GuessChar {
    pub knowledge: CharKnowledge,
    pub ch: Option<char>,
}

impl GuessChar {
    /// Sets the `CharKnowledge` for this guess, as long as it has some `char`.
    pub fn set_knowledge(&mut self, knowledge: CharKnowledge) {
        if let Some(_) = self.ch {
            self.knowledge = knowledge;
        }
    }

    pub fn set_ch(&mut self, ch: Option<char>) {
        self.ch = ch;
        if let None = ch {
            self.knowledge = CharKnowledge::Unknown;
        }
    }
}

pub struct GuessStr<const N: usize> {
    guesses: Vec<GuessChar>,
    active_ch: Option<usize>, // TODO: "active" is a UI aspect, move this to a wrapper in tui.rs
}

impl<const N: usize> GuessStr<N> {
    pub fn new() -> Self {
        let mut result = GuessStr {
            guesses: Vec::with_capacity(N),
            active_ch: None,
        };
        for _ in 0..N {
            result.guesses.push(GuessChar {
                knowledge: CharKnowledge::Unknown,
                ch: None,
            })
        }
        return result;
    }

    pub fn chars(&self) -> Iter<'_, GuessChar> {
        return self.guesses.iter();
    }

    pub fn guesses(&self) -> &Vec<GuessChar> {
        &self.guesses
    }

    pub fn active_ch(&self) -> Option<usize> {
        self.active_ch
    }

    // TODO move this to tui, too!
    pub fn cycle_guess_knowledge(&mut self, up: bool) {
        if let Some(active) = self.active_ch {
            let guess = self.guesses.get_mut(active).expect("out of bounds");
            let curr_knowledge = guess.knowledge;
            let next_idx = incr_usize(curr_knowledge as usize, CharKnowledge::COUNT, up, WRAP);
            let next = CharKnowledge::from_repr(next_idx)
                .expect(&format!("out of range for {}", next_idx));
            guess.set_knowledge(next);
        }
    }

    pub fn move_active_ch(&mut self, right: bool) {
        if self.guesses.is_empty() {
            return;
        }
        match self.active_ch {
            None => {
                if right {
                    self.active_ch = Some(self.guesses.len() - 1)
                } else {
                    self.active_ch = Some(0)
                }
            }
            Some(curr) => {
                self.active_ch = Some(incr_usize(curr, self.guesses.len(), right, WRAP));
            }
        }
    }

    pub fn set_active_ch(&mut self, ch: char) {
        if !ch.is_ascii_alphabetic() {
            return;
        }
        self.set_active_ch_direct(Some(ch.to_ascii_uppercase()));
        self.move_active_ch(true);
    }

    pub fn unset_active_ch(&mut self) {
        if let Some(idx) = self.active_ch {
            let active = self.guesses.get_mut(idx).expect("out of bounds");
            if let None = active.ch {
                self.move_active_ch(false);
            }
            self.set_active_ch_direct(None)
        }
    }

    fn set_active_ch_direct(&mut self, ch: Option<char>) {
        if let Some(idx) = self.active_ch {
            let active = self.guesses.get_mut(idx).expect("out of bounds");
            active.set_ch(ch);
        }
    }
}

pub struct GuessGrid<const N: usize, const R: usize> {
    guesses: Vec<GuessStr<N>>,
    active_row: usize, // TODO "active" is a UI concern, move this there
}

impl<const N: usize, const R: usize> GuessGrid<N, R> {
    pub fn new() -> Self {
        let mut result = GuessGrid {
            guesses: Vec::with_capacity(R),
            active_row: 0,
        };
        for _ in 0..R {
            result.guesses.push(GuessStr::new())
        }
        result.guesses[0].active_ch = Some(0);
        return result;
    }

    pub fn rows(&self) -> Iter<'_, GuessStr<N>> {
        return self.guesses.iter();
    }

    pub fn active_row(&self) -> usize {
        self.active_row
    }

    pub fn guesses(&self) -> &Vec<GuessStr<{ N }>> {
        &self.guesses
    }

    pub fn active_guess(&mut self) -> &GuessStr<N> {
        &self.guesses[self.active_row]
    }

    pub fn active_guess_mut(&mut self) -> &mut GuessStr<N> {
        &mut self.guesses[self.active_row]
    }

    pub fn set_active_char_on_active_row(&mut self, active: Option<usize>) {
        self.guesses[self.active_row].active_ch = active;
    }

    pub fn increment_active(&mut self) {
        self.active_row += 1
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

const WRAP: bool = true;
const NO_WRAP: bool = false;
