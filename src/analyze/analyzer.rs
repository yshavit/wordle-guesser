use crate::analyze::pattern::PatternBasedAnalyzer;
use crate::analyze::position_chars::CharPositionScorer;
use crate::analyze::scored_chars::CharScorer;
use crate::analyze::simple_analyzers::{CharFrequencies, Random, WordFrequencies};
use crate::word_list::WordList;
use std::cmp::Ordering;

pub trait Analyzer<const N: usize> {
    fn name(&self) -> String;
    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>>;
}

pub fn standard_suite<const N: usize>() -> Vec<Box<dyn Analyzer<N>>> {
    vec![
        Box::new(CharFrequencies {}),
        // Box::new(AlphabeticalOrder { ascending: true }),
        // Box::new(AlphabeticalOrder { ascending: false }),
        Box::new(CharScorer {
            double_count_freq: false,
        }),
        Box::new(CharScorer {
            double_count_freq: true,
        }),
        Box::new(WordFrequencies {}),
        Box::new(CharPositionScorer {}),
        Box::new(PatternBasedAnalyzer { limit: 10000 }),
        Box::new(Random {}),
    ]
}

#[derive(PartialEq)]
pub struct ScoredWord<'a> {
    pub word: &'a str,
    pub score: f64,
}

impl<'a> ScoredWord<'a> {
    pub fn normalize_scores(words: &mut Vec<ScoredWord<'a>>) {
        let mut min_score: f64 = f64::INFINITY;
        let mut max_score: f64 = f64::NEG_INFINITY;
        for word in words.iter() {
            min_score = min_score.min(word.score);
            max_score = max_score.max(word.score);
        }
        if min_score < max_score {
            for word in words {
                word.score = 100.0 * (word.score - min_score) / (max_score - min_score);
            }
        } else if min_score.is_finite() && min_score == max_score {
            // This happens if there was only one word; min and max are equal
            for word in words {
                word.score = 100.0;
            }
        }
    }
}

impl<'a> PartialOrd for ScoredWord<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Initially compare by score, in reverse order. Then by word, in order.
        Some(self.cmp(other))
    }
}

impl<'a> Ord for ScoredWord<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.score.total_cmp(&self.score) {
            Ordering::Equal => self.word.cmp(other.word),
            ne => ne,
        }
    }
}

impl<'a> Eq for ScoredWord<'a> {}
