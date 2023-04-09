use crate::guess::guesses::{GuessGrid, GuessStr};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use strum::{EnumCount, FromRepr};

#[derive(Copy, Clone, PartialEq, EnumCount, FromRepr)]
pub enum CharKnowledge {
    Unknown,
    WrongPosition,
    Correct,
    Missing,
}

impl Default for CharKnowledge {
    fn default() -> Self {
        CharKnowledge::Unknown
    }
}

pub struct KnownWordConstraints<const N: usize> {
    fully_known: [Option<char>; N],
    wrong_positions: Vec<HashSet<char>>,
    missing: HashSet<char>,
    letters_count: KnowledgePerLetter,
}

impl<const N: usize> KnownWordConstraints<N> {
    pub fn is_word_possible(&self, word: &str) -> bool {
        // First, check all the positional info.
        for (idx, word_ch) in word.chars().enumerate() {
            // If we know this idx has to contain known_ch, but it doesn't, then return false.
            if let Some(known_ch) = self.fully_known[idx] {
                if word_ch != known_ch {
                    return false;
                }
            }
            // If this idx can't contain a given char, but it does, then return false.
            if self.wrong_positions[idx].contains(&word_ch) {
                return false;
            }
        }
        // Now the counts
        let mut chars_count = HashMap::with_capacity(N);
        for word_char in word.chars() {
            *chars_count.entry(word_char).or_insert(0) += 1;
        }
        for (known_ch, known_count) in self.letters_count.0.iter() {
            let count_in_word = chars_count.get(known_ch).unwrap_or(&0);
            if count_in_word < &known_count.at_least {
                return false;
            }
            if let Some(no_more_than) = known_count.no_more_than {
                if count_in_word > &no_more_than {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn from_grid<const R: usize>(grid: &GuessGrid<N, R>) -> Self {
        // initial info
        let mut result = KnownWordConstraints {
            fully_known: [None; N],
            wrong_positions: Vec::with_capacity(N),
            missing: HashSet::new(),
            letters_count: KnowledgePerLetter::new(N),
        };
        for _ in 0..N {
            result.wrong_positions.push(HashSet::new());
        }

        for row in grid.rows() {
            result.add_row(row);
            let row_counts = KnowledgePerLetter::from(row);
            result.letters_count.add(&row_counts);
        }
        grid.rows().for_each(|r| result.add_row(r));

        return result;
    }

    pub fn add_row(&mut self, str: &GuessStr<N>) {
        for (idx, guess_ch) in str.chars().enumerate() {
            let Some(ch) = guess_ch.ch() else {
                continue;
            };
            match guess_ch.knowledge() {
                CharKnowledge::Correct => self.fully_known[idx] = Some(ch),
                CharKnowledge::WrongPosition => {
                    self.wrong_positions.get_mut(idx).unwrap().insert(ch);
                }
                CharKnowledge::Missing => {
                    self.missing.insert(ch);
                }
                CharKnowledge::Unknown => continue,
            };
        }
    }
}

/// A description of how many times a letter occurs within a given word. We only need this for
/// letters that we know exist at least once, so the `at_least` field is a plain usize. On the other
/// hand, we may not (and often will not) know the maximum of how many times a letter appears, so
/// `no_more_than` is an optional.
///
/// For example, if we guessed `F O O B R`, and got a "wrong position" for the first `O` but a
/// "missing" for the second `O`, then we know that `LetterCount { at_least: 1, no_more_than: 1 }`.
#[derive(Default)]
struct LetterKnowledge {
    at_least: usize,
    no_more_than: Option<usize>,
}

struct KnowledgePerLetter(HashMap<char, LetterKnowledge>);

impl KnowledgePerLetter {
    fn new(capacity: usize) -> Self {
        KnowledgePerLetter(HashMap::with_capacity(capacity))
    }

    fn from<const N: usize>(string: &GuessStr<N>) -> KnowledgePerLetter {
        let mut result = Self::new(N);

        for guess in string.chars() {
            let Some(mut ch) = guess.ch() else {
                continue;
            };
            match guess.knowledge() {
                CharKnowledge::WrongPosition | CharKnowledge::Correct => {
                    ch = ch.to_ascii_uppercase();
                    let count = result.0.entry(ch).or_insert(Default::default());
                    count.at_least += 1;
                }
                CharKnowledge::Missing => {} // will be handled below
                CharKnowledge::Unknown => {}
            }
        }
        for guess in string.chars() {
            let Some(mut ch) = guess.ch() else {
                continue;
            };
            if guess.knowledge() == CharKnowledge::Missing {
                ch = ch.to_ascii_uppercase();
                let count = result.0.entry(ch).or_insert(Default::default());
                count.no_more_than = Some(count.at_least);
            }
        }
        return result;
    }

    fn add(&mut self, other: &KnowledgePerLetter) {
        for (ch, other_count) in &other.0 {
            let entry = self.0.entry(*ch);
            let my_count = entry.or_insert_with(|| LetterKnowledge::default());
            let at_least = max(my_count.at_least, other_count.at_least);
            let no_more_than = match (my_count.no_more_than, other_count.no_more_than) {
                (Some(my_ceil), Some(other_ceil)) => Some(min(my_ceil, other_ceil)),
                (ceil @ Some(_), None) => ceil,
                (None, ceil @ Some(_)) => ceil,
                (None, None) => None,
            };
            if let Some(ceil) = no_more_than {
                if at_least > ceil {
                    // We got conflicting data! Just ignore this letter
                    continue;
                }
            }
            my_count.at_least = at_least;
            my_count.no_more_than = no_more_than;
        }
    }
}
