use std::collections::{HashSet};

pub fn uniq_chars(word: &str) -> HashSet<char> {
    let mut unique_chars = HashSet::with_capacity(word.len());
    for ch in word.chars() {
        unique_chars.insert(ch.to_ascii_uppercase());
    }
    unique_chars
}

pub fn chars_count<I>(chars: I) -> CharsCount
where
    I: Iterator<Item = char>,
{
    let mut chars_count = CharsCount::default();
    for ch in chars.into_iter() {
        chars_count.increment(ch);
    }
    chars_count
}

#[derive(Default)]
pub struct CharsCount {
    counts: [u32; ('Z' as usize - 'A' as usize) + 1],
}

impl CharsCount {

    pub fn get(&self, ch: char) -> u32 {
        if !ch.is_ascii_alphabetic() {
            return 0;
        }
        let ch = ch.to_ascii_uppercase();
        return self.counts[ch as usize - 'A' as usize];
    }

    pub fn get_mut(&mut self, ch: char) -> Option<&mut u32> {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        let ch = ch.to_ascii_uppercase();
        return Some(&mut self.counts[ch as usize - 'A' as usize]);
    }

    pub fn increment(&mut self, ch: char) {
        if let Some(count) = self.get_mut(ch) {
            *count += 1;
        }
    }

    pub fn decrement(&mut self, ch: char) {
        if let Some(count) = self.get_mut(ch) {
            *count -= 1;
        }
    }
}
