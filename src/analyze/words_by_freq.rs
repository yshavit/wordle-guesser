use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::word_list::{WordFreq, WordList};

pub struct WordsByFrequency {}

impl<const N: usize> Analyzer<N> for WordsByFrequency {
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
