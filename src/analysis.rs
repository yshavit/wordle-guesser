use std::collections::{BTreeMap, HashMap, HashSet};
use crate::word_list::WordList;

pub struct CharCounts<'a, const N: usize> {
    occurrence_by_char: BTreeMap<char, u32>,
    word_count_by_char: HashMap<char, u32>,
    total_chars: u32,
    words_count: u32,
    words_list: &'a WordList<N>,
}

impl<'a, const N: usize> CharCounts<'a, N> {
    pub fn new(words_list: &'a WordList<N>) -> Self {
        let mut char_counts = Self {
            occurrence_by_char: BTreeMap::new(),
            word_count_by_char: HashMap::new(),
            total_chars: 0,
            words_count: 0,
            words_list
        };
        for word_and_freq in words_list.words() {
            let word = &word_and_freq.word;
            for ch in word.chars() {
                *char_counts.occurrence_by_char.entry(ch).or_insert(0) += 1;
                char_counts.total_chars += 1;
            }
            for ch in uniq_chars(word) {
                *char_counts.word_count_by_char.entry(ch).or_insert(0) += 1;
            }
            char_counts.words_count += 1;
        }
        char_counts
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
        let Some(char_words) = self.word_count_by_char.get(ch) else {
            return 0.0;
        };
        let ratio = (*char_words as f64) / (self.words_count as f64);
        1.0 - (0.5 - ratio).abs()
    }

    fn all_char_scores(&self) -> HashMap<char, f64> {
        let uniq_chars = self.occurrence_by_char.keys();
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
            for ch in uniq_chars(word) {
                score += all_char_scores.get(&ch).unwrap_or(&0.0)
            }
            score *= word_freq.freq as f64;
            result.push((&word as &str, score))
        }
        result.sort_by(|a, b| b.1.total_cmp(&a.1));
        result
    }
}

fn uniq_chars(word: &str) -> HashSet<char> {
    let mut unique_chars = HashSet::with_capacity(word.len());
    for ch in word.chars() {
        unique_chars.insert(ch);
    }
    unique_chars
}
