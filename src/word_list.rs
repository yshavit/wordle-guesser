use crate::guess::known_word_constraints::KnownWordConstraints;
use crate::word_list::Iter::ForFiltered;
use crate::word_list::WordList::{Empty, Filtered, Reified};
use bitvec::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct WordFreq {
    pub word: String,
    pub freq: u32,
}

#[derive(Clone)]
pub enum WordList<const N: usize> {
    Empty,
    Reified {
        words: Rc<Vec<WordFreq>>,
    },
    Filtered {
        words: Rc<Vec<WordFreq>>,
        allowed: BitVec<usize, Lsb0>,
    },
}

impl<const N: usize> WordList<N> {
    pub fn empty() -> Self {
        Empty
    }

    pub fn get_embedded_std() -> Self {
        Self::get_embedded(7_500)
    }

    pub fn get_embedded(limit: usize) -> Self {
        let file = include_str!("words-5chars.txt");
        let mut words = Vec::with_capacity(file.chars().filter(|c| c == &'\n').count());
        for line in file.split("\n") {
            let Some((word, freq_str)) = line.split_once("\t") else {
                continue;
            };
            if word.len() != N {
                continue;
            }
            let Ok(freq) = freq_str.parse::<u32>() else {
                continue;
            };
            words.push(WordFreq {
                word: word.to_ascii_uppercase(),
                freq,
            });
            if words.len() >= limit {
                break;
            }
        }
        return Reified {
            words: Rc::new(words),
        };
    }

    pub fn filter(&mut self, knowledge: &KnownWordConstraints<N>) {
        match self {
            Empty => {}
            Reified { words } => {
                let existing = words.as_ref();
                let mut new = existing.clone();
                new.retain(|word| knowledge.is_word_possible(&word.word));
                *words = Rc::new(new);
            }
            Filtered { words, allowed } => {
                let mut remove: BitVec<usize, Lsb0> = BitVec::repeat(false, allowed.len());
                for idx in allowed.iter_ones() {
                    let word = &words[idx].word;
                    if !knowledge.is_word_possible(word) {
                        remove.set(idx, true);
                    }
                }
                for idx in remove.iter_ones() {
                    allowed.set(idx, false);
                }
            }
        }
    }

    pub fn filter_preview(&self, knowledge: &KnownWordConstraints<N>) -> Self {
        let mut new = match self {
            Empty => Empty,
            Reified { words } => Filtered {
                words: words.clone(),
                allowed: BitVec::repeat(true, words.as_ref().len()),
            },
            Filtered { words, allowed } => Filtered {
                words: words.clone(),
                allowed: allowed.clone(),
            },
        };
        new.filter(knowledge);
        new
    }

    pub fn words(&self) -> Iter {
        match self {
            Empty => Iter::ForEmpty,
            Reified { words } => Iter::ForReified {
                iter: words.iter(),
                total_length: words.len(),
            },
            Filtered { words, allowed } => ForFiltered {
                all_words: words,
                allowed_words: allowed.iter_ones(),
                total_length: allowed.count_ones(),
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Empty => 0,
            Reified { words } => words.as_ref().len(),
            Filtered { allowed, .. } => allowed.count_ones(),
        }
    }
}

pub enum Iter<'a> {
    ForEmpty,
    ForReified {
        iter: std::slice::Iter<'a, WordFreq>,
        total_length: usize,
    },
    ForFiltered {
        all_words: &'a Vec<WordFreq>,
        allowed_words: bitvec::slice::IterOnes<'a, usize, Lsb0>,
        total_length: usize,
    },
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a WordFreq;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::ForEmpty => None,
            Iter::ForReified { iter, .. } => iter.next(),
            ForFiltered {
                all_words,
                allowed_words,
                ..
            } => allowed_words.next().map(|idx| all_words.get(idx)).flatten(),
        }
    }
}

impl<'a> Iter<'a> {
    pub fn total_length(&self) -> usize {
        match self {
            Iter::ForEmpty => 0,
            Iter::ForReified { total_length, .. } => *total_length,
            ForFiltered { total_length, .. } => *total_length,
        }
    }
}
