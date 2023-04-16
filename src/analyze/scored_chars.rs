use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::char_stats::CharCounts;
use crate::analyze::util;
use crate::word_list::WordList;
use std::collections::HashMap;

pub struct CharScorer<const N: usize> {
    /// An individual char's score is basically how well it bisects all of the word; this metric
    /// inherently includes the char's frequency in that word list. Then, a word's overall score can
    /// be one of two things:
    ///
    /// 1. The sum of its chars' benefits; or
    /// 2. The sum of the chars' benefits, weighted to how likely we are to actually see that char.
    ///
    /// The second one of those seems useful at first: if a char is very beneficial, but unlikely
    /// to be seen, then it's probably not great.
    ///
    /// But actually that double-counts the frequency, since the benefit calculation itself included
    /// that frequency. Besides, if a char is rare, then it won't bisect the words list: so it won't
    /// be beneficial anyway!
    ///
    /// So, including that weight essentially double-counts the frequency, and in doing so makes
    /// this analysis much closer to just "the most frequent words".
    ///
    /// It's recommended to keep this value off.
    pub double_count_freq: bool,
}

impl<const N: usize> Analyzer<N> for CharScorer<N> {
    fn name(&self) -> String {
        if self.double_count_freq {
            "Scored Chars (2x-count freq)"
        } else {
            "Scored Chars (std)"
        }
        .to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let char_counts = CharCounts::new(words_list);
        let scorer = ScoredChars::new(&words_list, &char_counts, self.double_count_freq);
        scorer.all_word_scores()
    }
}

struct ScoredChars<'a, 'b, const N: usize> {
    char_score_includes_frequency: bool,
    counts: &'a CharCounts<N>,
    words_list: &'b WordList<N>,
}

impl<'a, 'b, const N: usize> ScoredChars<'a, 'b, N> {
    pub fn new(
        words_list: &'b WordList<N>,
        char_counts: &'a CharCounts<N>,
        double_count_freq: bool,
    ) -> Self {
        ScoredChars {
            counts: char_counts,
            words_list,
            char_score_includes_frequency: double_count_freq,
        }
    }

    /// This is the char's likely benefit, where:
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

    fn all_word_scores(&self) -> Vec<ScoredWord<'b>> {
        let all_words = self.words_list.words();
        let all_char_scores = self.all_char_scores();

        let mut result = Vec::with_capacity(all_words.total_length());
        for word_freq in all_words {
            let word = &word_freq.word;
            let mut score = 0.0;
            for ch in util::uniq_chars(word) {
                score += all_char_scores.get(&ch).unwrap_or(&0.0)
            }
            if self.char_score_includes_frequency {
                score *= word_freq.freq as f64;
            }
            result.push(ScoredWord { word, score });
        }
        result
    }
}
