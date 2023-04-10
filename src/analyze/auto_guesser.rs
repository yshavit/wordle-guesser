use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::auto_guesser::GuessResult::{Failure, Success};
use crate::guess::guesses::{GuessGrid, GuessStr};
use crate::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use crate::word_list::WordList;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::ops::DerefMut;

pub struct AutoGuesser<const N: usize, const R: usize> {
    pub answer_words: Vec<String>,
    pub words_list: WordList<N>,
    pub analyzers: Vec<Analyzer<N>>,
}

pub enum GuessResult {
    Success(Vec<String>),
    Failure,
}

pub struct AnalyzerGuessResult {
    pub name: String,
    pub result: GuessResult,
}

pub struct ResultsByWord {
    pub answer: String,
    pub analyzer_results: Vec<AnalyzerGuessResult>,
}

impl<const N: usize, const R: usize> AutoGuesser<N, R> {
    pub fn guess_all(self) -> Vec<ResultsByWord> {
        let mut all_results = Vec::with_capacity(self.answer_words.len());
        for answer in self.answer_words {
            if answer.len() != N {
                continue;
            }
            let mut results_by_analyzer = Vec::with_capacity(self.analyzers.len());
            for analyzer in &self.analyzers {
                let result = Self::guess_one(&self.words_list, &answer, analyzer);
                results_by_analyzer.push(AnalyzerGuessResult {
                    name: analyzer.name.to_string(), // TODO can borrow, with some lifetime trickery
                    result,
                });
            }
            all_results.push(ResultsByWord {
                answer,
                analyzer_results: results_by_analyzer,
            })
        }
        all_results
    }

    pub fn guess_one(
        words_list: &WordList<N>,
        answer: &str,
        analyzer: &Analyzer<N>,
    ) -> GuessResult {
        let mut grid = GuessGrid::<N, R>::new();
        let mut guesses = Vec::with_capacity(R);
        let mut possible_words = words_list.filter_preview(&KnownWordConstraints::empty());
        let answer_upper = answer.to_ascii_uppercase();
        for guess_num in 0..R {
            possible_words.filter(&KnownWordConstraints::from_grid(&grid));
            let mut scores: Vec<ScoredWord> = (analyzer.func)(&possible_words);
            scores.sort();
            let Some(&ScoredWord{word: best_guess, ..}) = scores.first() else {
                return Failure;
            };
            guesses.push(best_guess.to_string());
            if best_guess == answer_upper {
                return Success(guesses.into_iter().map(|s| s.to_string()).collect());
            }
            Self::enter_guess(best_guess, grid.guess_mut(guess_num), &answer_upper);
        }
        Failure
    }

    fn enter_guess(guess: &str, output: &mut GuessStr<N>, answer: &str) {
        let mut chars_count = HashMap::with_capacity(N);
        let mut answer_chars = ['\x00'; N];
        for (idx, mut answer_ch) in answer.chars().enumerate() {
            answer_ch = answer_ch.to_ascii_uppercase();
            *chars_count.entry(answer_ch).or_insert(0).deref_mut() += 1;
            answer_chars[idx] = answer_ch;
        }
        // Do this in two passes: first to find all the chars that are in the right position,
        // and then the rest.
        for (idx, mut guess_str_char) in guess.chars().enumerate() {
            guess_str_char = guess_str_char.to_ascii_uppercase();
            let guess_ch = output.guess_mut(idx);
            guess_ch.set_ch(guess_str_char);
            if guess_str_char == answer_chars[idx] {
                guess_ch.set_knowledge(CharKnowledge::Correct);
                *chars_count.entry(guess_str_char).or_insert(0).deref_mut() -= 1;
            }
        }
        // Now, chars that might be in the wrong position
        for (idx, mut guess_str_char) in guess.chars().enumerate() {
            guess_str_char = guess_str_char.to_ascii_uppercase();
            let guess_ch = output.guess_mut(idx);
            if guess_ch.knowledge() == CharKnowledge::Correct {
                continue; // already handled above
            }
            match chars_count.entry(guess_str_char) {
                Entry::Occupied(mut entry) => {
                    let count = entry.get_mut();
                    if *count <= 0 {
                        guess_ch.set_knowledge(CharKnowledge::Missing);
                    } else if *count == 1 {
                        guess_ch.set_knowledge(CharKnowledge::WrongPosition);
                        entry.remove();
                    } else {
                        *count -= 1;
                    }
                }
                Entry::Vacant(_) => guess_ch.set_knowledge(CharKnowledge::Missing),
            }
        }
    }
}
