use std::collections::{HashMap, HashSet};

pub fn uniq_chars(word: &str) -> HashSet<char> {
    let mut unique_chars = HashSet::with_capacity(word.len());
    for ch in word.chars() {
        unique_chars.insert(ch.to_ascii_uppercase());
    }
    unique_chars
}

pub fn chars_count<I>(chars: I) -> HashMap<char, i32>
where
    I: Iterator<Item = char>,
{
    let mut chars_count = HashMap::with_capacity(26);
    for ch in chars.into_iter() {
        let ch_upper = ch.to_ascii_uppercase();
        *chars_count.entry(ch_upper).or_insert(0) += 1
    }
    chars_count
}
