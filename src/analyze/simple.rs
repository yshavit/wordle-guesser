use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::word_list::WordList;

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
