use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::util;
use crate::analyze::util::uniq_chars;
use crate::word_list::{WordFreq, WordList};

pub struct AlphabeticalOrder {
    pub ascending: bool,
}

impl<const N: usize> Analyzer<N> for AlphabeticalOrder {
    fn name(&self) -> String {
        if self.ascending {
            "Alphabetical  ︎↓"
        } else {
            "Alphabetical ↑"
        }
        .to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let mut words: Vec<ScoredWord<'a>> = words_list
            .words()
            .map(|w| ScoredWord {
                word: &w.word,
                score: 0.0,
            })
            .collect();
        words.sort();
        if self.ascending {
            words.reverse();
        }
        for (idx, word) in words.iter_mut().enumerate() {
            word.score = idx as f64;
        }
        words
    }
}

pub struct CharFrequencies {}

impl<const N: usize> Analyzer<N> for CharFrequencies {
    fn name(&self) -> String {
        "Char Freqs".to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let chars_count = util::chars_count(words_list.all_chars());
        words_list
            .words()
            .map(|WordFreq { word, .. }| {
                let score: u32 = uniq_chars(word)
                    .into_iter()
                    .map(|c| chars_count.get(c))
                    .sum();
                ScoredWord {
                    word,
                    score: score as f64,
                }
            })
            .collect()
    }
}

pub struct WordFrequencies {}

impl<const N: usize> Analyzer<N> for WordFrequencies {
    fn name(&self) -> String {
        "Most Common Words".to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        words_list
            .words()
            .map(|WordFreq { word, freq }| ScoredWord {
                word,
                score: *freq as f64,
            })
            .collect()
    }
}
