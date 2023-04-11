use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::util::uniq_chars;
use crate::word_list::{WordFreq, WordList};
use std::collections::HashMap;

pub struct CharFrequencies {}

impl<const N: usize> Analyzer<N> for CharFrequencies {
    fn name(&self) -> String {
        "Char Freqs".to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let mut chars_count = HashMap::with_capacity(26);
        for ch in words_list.words().flat_map(|w| w.word.chars()) {
            *chars_count.entry(ch).or_insert(0) += 1
        }

        words_list
            .words()
            .map(|WordFreq { word, .. }| {
                let score: i32 = uniq_chars(word)
                    .into_iter()
                    .map(|c| chars_count.get(&c).unwrap_or(&0))
                    .sum();
                ScoredWord {
                    word,
                    score: score as f64,
                }
            })
            .collect()
    }
}
