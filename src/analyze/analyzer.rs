use crate::word_list::WordList;
use std::cmp::Ordering;

pub struct Analyzer<const N: usize> {
    pub name: String,
    pub func: for<'a> fn(&'a WordList<N>) -> Vec<ScoredWord<'a>>,
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
