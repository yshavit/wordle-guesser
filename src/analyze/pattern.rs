use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::guess::known_word_constraints::CharKnowledge;
use crate::word_list::{WordFreq, WordList};
use bitvec::vec::BitVec;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use strum::EnumCount;
use crate::analyze::util::CharsCount;

const MAX_WORD_LEN_FOR_BITVEC: usize = 5;

pub struct PatternBasedAnalyzer<const N: usize> {
    pub limit: usize,
}

/// An implementation of roughly what I think the WorldBot uses
impl<const N: usize> Analyzer<N> for PatternBasedAnalyzer<N> {
    fn name(&self) -> String {
        format!("Pattern ({})", self.limit)
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        let words_list_copy = words_list.reify();

        let mut words_and_scores: Vec<(ScoredWord<'a>, f64)> = words_list
            .words()
            .take(self.limit)
            .map(|w| {
                (
                    ScoredWord {
                        word: &w.word,
                        score: Self::score_word(&w.word, &words_list_copy) as f64,
                    },
                    w.freq,
                )
            })
            .collect();

        let max_freq_and_score = words_and_scores.iter().fold((0, 0.0, 0.0), |acc, entry| {
            let (count, acc_freq, acc_score): (u32, f64, f64) = acc;
            let entry_freq = entry.1;
            let entry_score = entry.0.score;
            (
                count + 1,
                acc_freq.max(entry_freq),
                acc_score.max(entry_score),
            )
        });
        if max_freq_and_score.0 > 0 {
            let max_freq = max_freq_and_score.1;
            let max_score = max_freq_and_score.2;
            for mut entry in words_and_scores.iter_mut() {
                let normalized_freq = entry.1 / max_freq;
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
        if N <= MAX_WORD_LEN_FOR_BITVEC {
            Self::score_word_0::<BitBasedPatternSet<N>>(word, &all_words)
        } else {
            Self::score_word_0::<HashSetBasedPatternSet<N>>(word, &all_words)
        }
    }

    fn score_word_0<P: PatternSet<N>>(word: &str, all_words: &WordList<N>) -> usize {
        let mut patterns = P::new();
        let mut answer_arr = ['\x00'; N];
        let mut answer_chars_count = CharsCount::default();
        for WordFreq {
            word: if_answer, ..
        } in all_words.words()
        {
            answer_chars_count.reset_all();
            for (idx, ch) in if_answer.chars().enumerate() {
                answer_arr[idx] = ch;
                answer_chars_count.increment(ch);
            }
            patterns.add(&Self::pattern(word, &answer_arr, &mut answer_chars_count));
        }
        patterns.size()
    }

    fn pattern(guess: &str, answer: &[char; N], answer_chars_count: &mut CharsCount) -> Pattern<N> {
        let mut result = Pattern {
            knowledge: [CharKnowledge::Missing; N],
        };
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

#[derive(PartialEq, Eq, Clone, Copy)]
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

trait PatternSet<const N: usize> {
    fn new() -> Self;
    fn add(&mut self, pattern: &Pattern<N>);
    fn size(&self) -> usize;
}

struct HashSetBasedPatternSet<const N: usize> {
    patterns: HashSet<Pattern<N>>,
}

impl<const N: usize> PatternSet<N> for HashSetBasedPatternSet<N> {
    fn new() -> Self {
        Self {
            patterns: Default::default(),
        }
    }

    fn add(&mut self, pattern: &Pattern<N>) {
        self.patterns.insert(pattern.clone());
    }

    fn size(&self) -> usize {
        self.patterns.len()
    }
}

/// A `PatternSet` that treats each `Pattern` as a number, and uses a `BitArray` to keep track of
/// which "numbers" are in the set.
///
/// A `Pattern` is just an array of `CharKnowledge`, which is an enum. If we consider each enum
/// value a "digit" (and there are 4 of them), this means `Pattern` is basically an `N`-digit number
/// in base 4. For `N=5`, this is 1024 values.
struct BitBasedPatternSet<const N: usize> {
    patterns: BitVec,
    count: usize,
}

impl<const N: usize> PatternSet<N> for BitBasedPatternSet<N> {
    fn new() -> Self {
        Self {
            patterns: BitVec::repeat(false, CharKnowledge::COUNT.pow(N as u32)),
            count: 0,
        }
    }

    fn add(&mut self, pattern: &Pattern<N>) {
        let pattern_as_usize = Self::pattern_to_usize(&pattern);
        if !self.patterns.replace(pattern_as_usize, true) {
            self.count += 1;
        }
    }

    fn size(&self) -> usize {
         self.count
    }
}

impl<const N: usize> BitBasedPatternSet<N> {
    #[inline]
    fn pattern_to_usize(pattern: &Pattern<N>) -> usize {
        let mut result = 0;
        let mut position_factor = 1;
        for position in 0..N {
            let digit = pattern.knowledge[position] as usize;
            result += digit * position_factor;
            position_factor *= CharKnowledge::COUNT;
        }

        return result;
    }
}
