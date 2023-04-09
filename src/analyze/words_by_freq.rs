use crate::analyze::analyzer::ScoredWord;
use crate::word_list::{WordFreq, WordList};

pub fn words_by_frequency<const N: usize>(words_list: &WordList<N>) -> Vec<ScoredWord> {
    words_list
        .words()
        .iter()
        .map(|WordFreq { word, freq }| ScoredWord {
            word,
            score: *freq as f64,
        })
        .collect()
}
