use std::array::IntoIter;
use std::slice::Iter;

use crate::guess::known_word_constraints::CharKnowledge;

#[derive(Default, Clone, PartialEq, Eq)]
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

    /// Sets the char, as long as it's a valid char. Returns whether it was a valid char.
    ///
    /// If `ch` is a valid char, then this also resets `self`'s knowledge to `Unknown`, unless the
    /// new char is the same as the old one. In this case, the invocation is a no-op but still
    /// returns `true` (since the char was valid).
    pub fn set_ch(&mut self, ch: char) -> bool {
        if self.ch == Some(ch) {
            return true;
        }
        if !ch.is_ascii_alphabetic() {
            return false;
        }
        self.ch = Some(ch.to_ascii_uppercase());
        self.knowledge = CharKnowledge::Unknown;
        true
    }

    /// A row is filled out iff all of its chars are Some chars with non-Unknown knowledge.
    pub fn unset_ch(&mut self) -> Option<char> {
        let old = self.ch;
        self.ch = None;
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

    fn is_fully_filled(&self) -> bool {
        self.chars()
            .all(|&GuessChar { ch, knowledge }| ch.is_some() && knowledge != CharKnowledge::Unknown)
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

    /// Gets a list of all known chars, by position. This does not intelligently handle conflicting
    /// information (e.g., one row saying char 0 is 'A' and another saying it's 'B').
    pub fn known_chars(&self) -> [Option<char>; N] {
        let mut result = [None; N];

        // For now, don't c
        for row in self.rows() {
            if !row.is_fully_filled() {
                break;
            }
            for (idx, guess_ch) in row.chars().enumerate() {
                if guess_ch.knowledge == CharKnowledge::Correct {
                    result[idx] = guess_ch.ch;
                }
            }
        }

        result
    }
}

impl<const N: usize, const R: usize> IntoIterator for GuessGrid<N, R> {
    type Item = GuessStr<N>;
    type IntoIter = IntoIter<GuessStr<N>, R>;

    fn into_iter(self) -> Self::IntoIter {
        self.guesses.into_iter()
    }
}

fn init_array<T: Default, const N: usize>() -> [T; N] {
    [(); N].map(|_| T::default())
}
