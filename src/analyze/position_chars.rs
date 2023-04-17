use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::analyze::util::uniq_chars;
use crate::word_list::{WordFreq, WordList};
use std::collections::{HashMap, HashSet};

pub struct CharPositionScorer<const N: usize> {}

impl<const N: usize> Analyzer<N> for CharPositionScorer<N> {
    fn name(&self) -> String {
        "Char-Pos".to_string()
    }

    fn analyze<'a>(&self, words_list: &'a WordList<N>) -> Vec<ScoredWord<'a>> {
        // First, we want the individual char scores, per position. Each one of those is basically
        // "how close does this char in this position get to cutting all the words into even
        // thirds?"
        //
        // For that, we need three counts for each (char, position): one for Correct, one for
        // Missing, one for WrongPosition.
        let all_chars: HashSet<char> = words_list.words().flat_map(|w| w.word.chars()).collect();

        let mut position_counts = [(); N].map(|_| HashMap::new());
        for WordFreq { word, .. } in words_list.words() {
            let word_chars = uniq_chars(word);
            for (idx, word_char) in word.chars().enumerate() {
                for guess_char in &all_chars {
                    let mut counts = &mut position_counts[idx]
                        .entry(*guess_char)
                        .or_insert_with(|| CharPosCounts::default());
                    if guess_char == &word_char {
                        counts.correct += 1;
                    } else if word_chars.contains(*guess_char) {
                        counts.wrong_position += 1;
                    } else {
                        counts.missing += 1;
                    }
                }
            }
        }

        let position_scores: [HashMap<char, f64>; N] = position_counts.map(|counts_by_char| {
            counts_by_char
                .into_iter()
                .map(|(k, v)| (k, v.score()))
                .collect()
        });

        // Okay, now we need to use that to score each char-position
        words_list
            .words()
            .map(|WordFreq { word, .. }| {
                let pos_char_sum: f64 = word
                    .chars()
                    .enumerate()
                    .map(|(idx, ch)| position_scores[idx].get(&ch).unwrap_or(&0.0))
                    .sum();
                let score = pos_char_sum / (N as f64);
                ScoredWord { word, score }
            })
            .collect()
    }
}

#[derive(Default)]
struct CharPosCounts {
    wrong_position: usize,
    correct: usize,
    missing: usize,
}

impl CharPosCounts {
    fn score(&self) -> f64 {
        closeness_to_equal(
            self.wrong_position as f64,
            self.correct as f64,
            self.missing as f64,
        )
    }
}

// thank you chatgpt :-)
fn closeness_to_equal(a: f64, b: f64, c: f64) -> f64 {
    let abs_diff_ab = (a - b).abs();
    let abs_diff_bc = (b - c).abs();
    let abs_diff_ca = (c - a).abs();
    -(abs_diff_ab + abs_diff_bc + abs_diff_ca)
}
