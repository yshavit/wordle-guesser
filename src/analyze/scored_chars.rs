use crate::analyze::char_stats::CharCounts;
use crate::word_list::WordList;
use std::collections::HashMap;
use crate::analyze::util;

pub struct ScoredChars<'a, const N: usize> {
    counts: &'a CharCounts<N>,
    words_list: &'a WordList<N>,
}

impl<'a, const N: usize> ScoredChars<'a, N> {
    pub fn new(words_list: &'a WordList<N>, char_counts: &'a CharCounts<N>) -> Self {
        ScoredChars {
            counts: char_counts,
            words_list,
        }
    }

    /// (likelihood of presence) * (benefit), where:
    /// - likelihood of presence is just `(char_occurrence / total)`
    /// - benefit is how closely it comes to bisecting the set. Specifically:
    ///   - let `words_count` be the total number of words we know, and `char_words` be the number
    ///     of words that contain `ch`.
    ///   - let `ratio = char_words / word_count`
    ///   - we want something that gets higher the closer `ratio` is to `0.5`. So, how about:
    ///     `1 - abs(0.5 - ratio)`
    fn char_score(&self, ch: &char) -> f64 {
        let Some(char_words) = self.counts.word_count_by_char().get(ch) else {
            return 0.0;
        };
        let ratio = (*char_words as f64) / (self.counts.words_count() as f64);
        1.0 - (0.5 - ratio).abs()
    }

    fn all_char_scores(&self) -> HashMap<char, f64> {
        let uniq_chars = self.counts.occurrence_by_char().keys();
        let mut result = HashMap::with_capacity(uniq_chars.len());
        for ch in uniq_chars {
            result.insert(*ch, self.char_score(ch));
        }
        result
    }

    pub fn all_word_scores(&self) -> Vec<(&'a str, f64)> {
        let all_words = self.words_list.words();
        let all_char_scores = self.all_char_scores();

        let mut result = Vec::with_capacity(all_words.capacity());
        for word_freq in all_words {
            let word = &word_freq.word;
            let mut score = 0.0;
            for ch in util::uniq_chars(word) {
                score += all_char_scores.get(&ch).unwrap_or(&0.0)
            }
            score *= word_freq.freq as f64;
            result.push((&word as &str, score))
        }
        result.sort_by(|a, b| b.1.total_cmp(&a.1));
        result
    }
}
