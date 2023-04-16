use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::util;
use crate::guess::known_word_constraints::CharKnowledge;
use crate::word_list::{WordFreq, WordList};

pub struct PatternBasedAnalyzer<const N: usize> {
    pub limit: usize
}

/// An implementation of roughly what I think the WorldBot uses
impl<const N: usize> Analyzer<N> for PatternBasedAnalyzer<N> {
    fn name(&self) -> String {
        return "Pattern-Based".to_string();
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        words_list
            .words()
            .take(self.limit)
            .map(|w| ScoredWord {
                word: &w.word,
                score: Self::score_word(&w.word, words_list),
            })
            .collect()
    }
}

impl<const N: usize> PatternBasedAnalyzer<N> {
    fn score_word(word: &str, all_words: &WordList<N>) -> f64 {
        let mut cache = CharCountsCache::default();
        let mut patterns: HashSet<Pattern<N>> = HashSet::new();
        for WordFreq{word: if_answer, ..} in all_words.words() {
            patterns.insert(Self::pattern(word, if_answer, &mut cache));
        }
        patterns.len() as f64
    }

    fn pattern(guess: &str, answer: &str, char_counter: &mut CharCountsCache) -> Pattern<N> {
        let mut result = Pattern {
            knowledge: [CharKnowledge::Missing; N],
        };

        let mut answer_chars_count = char_counter.get(answer);

        // first, all the ones in the right position
        let answer_chars: Vec<char> = answer.chars().collect();
        for (idx, guess_ch) in guess.chars().enumerate() {
            let Some(answer_ch) = answer_chars.get(idx) else {
                continue
            };
            if &guess_ch == answer_ch {
                result.knowledge[idx] = CharKnowledge::Correct;
                *answer_chars_count.get_mut(&guess_ch).expect("missing char") -= 1;
            }
        }

        // now all the ones in the wrong position
        for (idx, guess_ch) in guess.chars().enumerate() {
            if let Some(remaining) = answer_chars_count.get_mut(&guess_ch) {
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
    knowledge: [CharKnowledge; N]
}

impl<const N: usize> Hash for Pattern<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for k in self.knowledge {
            state.write_usize(k as usize);
        }
    }
}

#[derive(Default)]
struct CharCountsCache {
    values: HashMap<String, HashMap<char, i32>>
}

impl CharCountsCache {
    fn get(&mut self, word: &str) -> HashMap<char, i32> {
        match self.values.get(word) {
            Some(existing) => existing.clone(),
            None => {
                let new = util::chars_count(word.chars());
                self.values.insert(word.to_string(), new.clone());
                new.clone()
            }
        }
    }
}
