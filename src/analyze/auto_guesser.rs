use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::auto_guesser::GuessResult::{Failure, Success};
use crate::guess::guesses::{GuessGrid, GuessStr};
use crate::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use crate::word_list::WordList;
use strum::Display;

use crate::analyze::util;

pub struct AutoGuesser<const N: usize, const R: usize> {
    pub answer_words: Vec<String>,
    pub words_list: WordList<N>,
    pub analyzers: Vec<Box<dyn Analyzer<N>>>,
}

#[derive(Display)]
pub enum GuessResult {
    Success,
    Failure,
}

pub struct AnalyzerGuessResult<const N: usize> {
    pub name: String,
    pub result: GuessResult,
    pub guesses: Vec<GuessStr<N>>,
}

pub struct ResultsByWord<const N: usize> {
    pub answer: String,
    pub analyzer_results: Vec<AnalyzerGuessResult<N>>,
}

impl<const N: usize, const R: usize> AutoGuesser<N, R> {
    pub fn guess_all(self) -> Vec<ResultsByWord<N>> {
        let mut all_results = Vec::with_capacity(self.answer_words.len());
        for answer in self.answer_words {
            if answer.len() != N {
                continue;
            }
            let mut results_by_analyzer = Vec::with_capacity(self.analyzers.len());
            for analyzer in &self.analyzers {
                let (result, guesses) =
                    Self::guess_one(&self.words_list, &answer, analyzer.as_ref());
                results_by_analyzer.push(AnalyzerGuessResult {
                    name: analyzer.name().to_string(), // TODO can borrow, with some lifetime trickery
                    result,
                    guesses,
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
        analyzer: &dyn Analyzer<N>,
    ) -> (GuessResult, Vec<GuessStr<N>>) {
        let mut grid = GuessGrid::<N, R>::new();
        let mut possible_words = words_list.filter_preview(&KnownWordConstraints::empty());
        let answer_upper = answer.to_ascii_uppercase();
        for guess_num in 0..R {
            possible_words.filter(&KnownWordConstraints::from_grid(&grid));
            let mut scores: Vec<ScoredWord> = analyzer.analyze(&possible_words);
            scores.sort();
            let Some(&ScoredWord{word: best_guess, ..}) = scores.first() else {
                return (Failure, grid.into_iter().take(guess_num).collect());
            };
            Self::enter_guess(best_guess, grid.guess_mut(guess_num), &answer_upper);
            if best_guess == answer_upper {
                return (Success, grid.into_iter().take(guess_num + 1).collect());
            }
        }
        (Failure, grid.into_iter().collect())
    }

    fn enter_guess(guess: &str, output: &mut GuessStr<N>, answer: &str) {
        let mut chars_count = util::chars_count(answer.chars());
        let mut answer_chars = ['\x00'; N];
        for (idx, mut answer_ch) in answer.chars().enumerate() {
            answer_ch = answer_ch.to_ascii_uppercase();
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
                chars_count.decrement(guess_str_char);
            }
        }
        // Now, chars that might be in the wrong position
        for (idx, mut guess_str_char) in guess.chars().enumerate() {
            guess_str_char = guess_str_char.to_ascii_uppercase();
            let guess_ch = output.guess_mut(idx);
            if guess_ch.knowledge() == CharKnowledge::Correct {
                continue; // already handled above
            }
            if let Some(count) = chars_count.get_mut(guess_str_char) {
                if *count <= 0 {
                    guess_ch.set_knowledge(CharKnowledge::Missing);
                } else {
                    guess_ch.set_knowledge(CharKnowledge::WrongPosition);
                    *count -= 1;
                }
            }
        }
    }
}
