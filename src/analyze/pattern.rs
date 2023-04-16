use std::cmp::max;
use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::util;
use crate::guess::known_word_constraints::CharKnowledge;
use crate::word_list::{WordFreq, WordList};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub struct PatternBasedAnalyzer<const N: usize> {
    pub limit: usize,
}

/// An implementation of roughly what I think the WorldBot uses
impl<const N: usize> Analyzer<N> for PatternBasedAnalyzer<N> {
    fn name(&self) -> String {
        format!("Pattern ({})", self.limit)
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let mut words_and_scores: Vec<(ScoredWord<'a>, u32)> = words_list
            .words()
            .take(self.limit)
            .map(|w| (
                ScoredWord{word: &w.word, score: Self::score_word(&w.word, words_list) as f64},
                w.freq))
            .collect();

        let max_freq_and_score = words_and_scores.iter().fold((0, 0, 0.0), |acc, entry| {
            let (count, acc_freq, acc_score): (u32, u32, f64) = acc;
            let entry_freq = entry.1;
            let entry_score = entry.0.score;
            (count + 1, max(acc_freq, entry_freq), acc_score.max(entry_score))
        });
        if max_freq_and_score.0 > 0 {
            let max_freq = max_freq_and_score.1 as f64;
            let max_score = max_freq_and_score.2 as f64;
            for mut entry in words_and_scores.iter_mut() {
                let normalized_freq = (entry.1 as f64) / max_freq;
                let normalized_score = entry.0.score / max_score;
                entry.0.score = normalized_score * 10.0 + normalized_freq;
            }
        }

        words_and_scores
            .into_iter()
            .map(|(scored_word, _)| scored_word)
            .collect()
    }
}

impl<const N: usize> PatternBasedAnalyzer<N> {
    fn score_word(word: &str, all_words: &WordList<N>) -> usize {
        let mut patterns: HashSet<Pattern<N>> = HashSet::new();
        let mut answer_arr = ['\x00'; N];
        for WordFreq {
            word: if_answer, ..
        } in all_words.words()
        {
            for (idx, ch) in if_answer.chars().enumerate() {
                answer_arr[idx] = ch;
            }
            patterns.insert(Self::pattern(word, &answer_arr));
        }
        patterns.len()
    }

    fn pattern(guess: &str, answer: &[char; N]) -> Pattern<N> {
        let mut result = Pattern {
            knowledge: [CharKnowledge::Missing; N],
        };

        let mut answer_chars_count = util::chars_count(answer.iter().map(|c| *c));

        // first, all the ones in the right position
        for (idx, guess_ch) in guess.chars().enumerate() {
            if guess_ch == answer[idx] {
                result.knowledge[idx] = CharKnowledge::Correct;
                answer_chars_count.decrement(guess_ch);
            }
        }

        // now all the ones in the wrong position
        for (idx, guess_ch) in guess.chars().enumerate() {
            if let Some(remaining) = answer_chars_count.get_mut(guess_ch) {
                if *remaining > 0 {
                    result.knowledge[idx] = CharKnowledge::WrongPosition;
                    *remaining -= 1;
                }
            }
        }

        result
    }
}

#[derive(PartialEq, Eq)]
struct Pattern<const N: usize> {
    knowledge: [CharKnowledge; N],
}

impl<const N: usize> Hash for Pattern<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for k in self.knowledge {
            state.write_usize(k as usize);
        }
    }
}
