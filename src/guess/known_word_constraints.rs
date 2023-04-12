use crate::guess::guesses::{GuessGrid, GuessStr};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use strum::{EnumCount, FromRepr};

#[derive(Copy, Clone, PartialEq, Eq, EnumCount, FromRepr)]
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

#[derive(PartialEq, Eq, Debug)]
pub struct KnownWordConstraints<const N: usize> {
    fully_known: [Option<char>; N],
    wrong_positions: [HashSet<char>; N],
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

    pub fn empty() -> Self {
        KnownWordConstraints {
            fully_known: [None; N],
            wrong_positions: [(); N].map(|_| HashSet::default()),
            missing: HashSet::new(),
            letters_count: KnowledgePerLetter::new(N),
        }
    }

    pub fn from_grid<const R: usize>(grid: &GuessGrid<N, R>) -> Self {
        let mut result = Self::empty();

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
#[derive(Default, PartialEq, Eq, Debug)]
struct LetterKnowledge {
    at_least: usize,
    no_more_than: Option<usize>,
}

#[derive(Default, PartialEq, Eq, Debug)]
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::guess::guesses::{GuessGrid, GuessStr};
    use std::collections::HashMap;

    #[test]
    fn create_knowledge_per_letter() {
        struct Case<const N: usize> {
            given: [(char, CharKnowledge); N],
            expect: HashMap<char, LetterKnowledge>,
        }
        let cases = vec![
            Case {
                given: [
                    ('I', CharKnowledge::Missing),
                    ('R', CharKnowledge::Missing),
                    ('A', CharKnowledge::Correct),
                    ('T', CharKnowledge::Missing),
                    ('E', CharKnowledge::Missing),
                ],
                #[rustfmt::skip]
                expect: vec![
                    ('I', LetterKnowledge { at_least: 0, no_more_than: Some(0) }),
                    ('R', LetterKnowledge { at_least: 0, no_more_than: Some(0) }),
                    ('A', LetterKnowledge { at_least: 1, no_more_than: None    }),
                    ('T', LetterKnowledge { at_least: 0, no_more_than: Some(0) }),
                    ('E', LetterKnowledge { at_least: 0, no_more_than: Some(0) }),
                ]
                .into_iter()
                .collect(),
            },
            Case {
                given: [
                    ('A', CharKnowledge::Missing),
                    ('B', CharKnowledge::WrongPosition),
                    ('C', CharKnowledge::WrongPosition),
                    ('B', CharKnowledge::Correct),
                    ('C', CharKnowledge::Missing),
                ],
                #[rustfmt::skip]
                expect: vec![
                    ('A', LetterKnowledge { at_least: 0, no_more_than: Some(0) }),
                    ('B', LetterKnowledge { at_least: 2, no_more_than: None    }),
                    ('C', LetterKnowledge { at_least: 1, no_more_than: Some(1) }),
                ]
                .into_iter()
                .collect(),
            },
        ];

        for case in cases {
            let mut guess_str: GuessStr<5> = GuessStr::new();
            write_chars(&mut guess_str, case.given);

            let expect = KnowledgePerLetter(case.expect);
            let actual = KnowledgePerLetter::from(&guess_str);

            assert_eq!(expect, actual);
        }
    }

    #[test]
    fn create_known_word_constraints() {
        let mut grid: GuessGrid<5, 6> = GuessGrid::new();
        write_chars(
            grid.guess_mut(0),
            [
                ('I', CharKnowledge::Missing),
                ('R', CharKnowledge::Missing),
                ('A', CharKnowledge::Correct),
                ('T', CharKnowledge::Missing),
                ('E', CharKnowledge::Missing),
            ],
        );
        write_chars(
            grid.guess_mut(1),
            [
                ('C', CharKnowledge::Missing),
                ('L', CharKnowledge::WrongPosition),
                ('A', CharKnowledge::Correct),
                ('S', CharKnowledge::Missing),
                ('H', CharKnowledge::Missing),
            ],
        );

        let actual = KnownWordConstraints::from_grid(&grid);

        let expected = KnownWordConstraints {
            fully_known: [None, None, Some('A'), None, None],
            wrong_positions: [
                HashSet::new(),
                HashSet::from(['L']),
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ],
            missing: HashSet::from(['I', 'R', 'T', 'E', 'C', 'S', 'H']),
            #[rustfmt::skip]
            letters_count: KnowledgePerLetter(vec![
                ('I', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('R', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('A', LetterKnowledge{at_least: 1, no_more_than: None }),
                ('T', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('E', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('C', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('L', LetterKnowledge{at_least: 1, no_more_than: None }),
                ('S', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
                ('H', LetterKnowledge{at_least: 0, no_more_than: Some(0) }),
            ].into_iter().collect()),
        };

        // These are covered by the big "assert_eq!(expected, actual)" at the end, but here's a more
        // granular breakdown.
        assert_eq!(expected.fully_known, actual.fully_known);
        assert_eq!(
            sorted_vec(&expected.fully_known),
            sorted_vec(&actual.fully_known)
        );
        assert_eq!(sorted_vec(&expected.missing), sorted_vec(&actual.missing));
        assert_eq!(
            entries(&expected.letters_count),
            entries(&actual.letters_count)
        );

        assert_eq!(expected, actual);

        assert!(actual.is_word_possible("QUALM"))
    }

    fn sorted_vec<I, T>(iterable: I) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
        T: Ord,
    {
        let mut vec: Vec<T> = iterable.into_iter().collect();
        vec.sort();
        vec
    }

    fn entries(knowledge_per_letter: &KnowledgePerLetter) -> Vec<(&char, &LetterKnowledge)> {
        let mut vec: Vec<(&char, &LetterKnowledge)> = knowledge_per_letter.0.iter().collect();
        vec.sort_by_key(|(ch, _)| *ch);
        vec
    }

    fn write_chars<const N: usize>(to: &mut GuessStr<N>, chars: [(char, CharKnowledge); N]) {
        for (idx, (ch, kn)) in chars.into_iter().enumerate() {
            let guess_ch = to.guess_mut(idx);
            guess_ch.set_ch(ch);
            guess_ch.set_knowledge(kn);
        }
    }
}
