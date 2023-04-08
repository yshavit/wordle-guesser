use std::collections::HashSet;

pub fn uniq_chars(word: &str) -> HashSet<char> {
    let mut unique_chars = HashSet::with_capacity(word.len());
    for ch in word.chars() {
        unique_chars.insert(ch);
    }
    unique_chars
}
