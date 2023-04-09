use std::slice::Iter;

use crate::guess::known_word_constraints::CharKnowledge;

#[derive(Default)]
pub struct GuessChar {
    knowledge: CharKnowledge,
    ch: Option<char>,
}

impl GuessChar {
    pub fn ch(&self) -> Option<char> {
        self.ch
    }

    pub fn knowledge(&self) -> CharKnowledge {
        self.knowledge
    }

    /// Sets the `CharKnowledge` for this guess, as long as it has some `char`.
    pub fn set_knowledge(&mut self, knowledge: CharKnowledge) {
        if let Some(_) = self.ch {
            self.knowledge = knowledge;
        }
    }

    pub fn set_ch(&mut self, ch: Option<char>) -> Option<char> {
        let old = self.ch;
        match ch {
            Some(ch) if ch.is_ascii_alphabetic() => {
                self.ch = Some(ch.to_ascii_uppercase());
            }
            None => self.ch = None,
            Some(_) => {} // keep it as-is
        }
        self.knowledge = CharKnowledge::Unknown;
        old
    }
}

pub struct GuessStr<const N: usize> {
    guesses: [GuessChar; N],
}

impl<const N: usize> GuessStr<N> {
    pub fn new() -> Self {
        GuessStr {
            guesses: init_array(),
        }
    }

    pub fn chars(&self) -> Iter<'_, GuessChar> {
        return self.guesses.iter();
    }

    pub fn guesses(&self) -> &[GuessChar; N] {
        &self.guesses
    }

    pub fn guess_mut(&mut self, idx: usize) -> &mut GuessChar {
        &mut self.guesses[idx]
    }
}

impl<const N: usize> Default for GuessStr<N> {
    fn default() -> Self {
        GuessStr::new()
    }
}


pub struct GuessGrid<const N: usize, const R: usize> {
    guesses: [GuessStr<N>; R],
}

impl<const N: usize, const R: usize> GuessGrid<N, R> {
    pub fn new() -> Self {
        GuessGrid {
            guesses: init_array(),
        }
    }

    pub fn rows(&self) -> Iter<'_, GuessStr<N>> {
        return self.guesses.iter();
    }

    pub fn guesses(&self) -> &[GuessStr<{ N }>] {
        &self.guesses
    }

    pub fn guess_mut(&mut self, idx: usize) -> &mut GuessStr<N> {
        &mut self.guesses[idx]
    }
}

fn init_array<T: Default, const N: usize>() -> [T; N] {
    [(); N].map(|_| T::default())
}
