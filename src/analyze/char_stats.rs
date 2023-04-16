use crate::analyze::util;
use crate::word_list::WordList;
use std::collections::{BTreeMap, HashMap};

pub struct CharCounts<const N: usize> {
    occurrence_by_char: BTreeMap<char, u32>,
    word_count_by_char: HashMap<char, u32>,
    total_chars: u32,
    words_count: u32,
}

impl<const N: usize> CharCounts<N> {
    pub fn new(words_list: &WordList<N>) -> Self {
        let mut char_counts = Self {
            occurrence_by_char: BTreeMap::new(),
            word_count_by_char: HashMap::new(),
            total_chars: 0,
            words_count: 0,
        };
        for word_and_freq in words_list.words() {
            let word = &word_and_freq.word;
            for ch in word.chars() {
                *char_counts.occurrence_by_char.entry(ch).or_insert(0) += 1;
                char_counts.total_chars += 1;
            }
            for ch in util::uniq_chars(word) {
                *char_counts.word_count_by_char.entry(ch).or_insert(0) += 1;
            }
            char_counts.words_count += 1;
        }
        char_counts
    }

    pub fn occurrence_by_char(&self) -> &BTreeMap<char, u32> {
        &self.occurrence_by_char
    }

    pub fn word_count_by_char(&self) -> &HashMap<char, u32> {
        &self.word_count_by_char
    }

    pub fn words_count(&self) -> u32 {
        self.words_count
    }
}
