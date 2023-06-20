use std::cmp::Ordering;
use crate::guess::known_word_constraints::KnownWordConstraints;
use crate::word_list::Iter::ForFiltered;
use crate::word_list::WordList::{Empty, Filtered, Reified};
use bitvec::prelude::*;
use std::collections::HashMap;
use std::iter::FlatMap;
use std::rc::Rc;
use std::str::Chars;
use std::usize;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct WordFreq {
    pub word: String,
    pub freq: f64,
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

#[derive(EnumIter)]
pub enum WordsFile {
    WGutenberg,
    Norvig,
    HermitDave,
}

impl WordsFile {
    pub fn get_embedded<const N: usize>(&self, limit: usize) -> WordList<N> {
        let file = self.get_file_contents();
        let mut words = Vec::with_capacity(file.chars().filter(|c| c == &'\n').count());
        for line in file.split("\n") {
            let Some((word, freq_str)) = line.split_once("\t") else {
                continue;
            };
            if word.len() != N {
                continue;
            }
            let Ok(freq) = freq_str.parse::<f64>() else {
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
}

impl WordsFile {
    fn get_file_contents(&self) -> &'static str {
        match self {
            WordsFile::WGutenberg => include_str!("words-5chars-wiktionary-gutenberg.txt"),
            WordsFile::Norvig => include_str!("words-5chars-norvig.txt"),
            WordsFile::HermitDave => include_str!("words-5chars-hermitdave.txt"),
        }
    }
}

impl<const N: usize> WordList<N> {
    pub fn empty() -> Self {
        Empty
    }

    pub fn std() -> Self {
        let limit = 10_000;
        let all_words_files = WordsFile::iter().map(|wl| wl.get_embedded(limit));
        Self::combine(all_words_files, limit)
    }

    pub fn combine<I>(items: I, limit: usize) -> Self
    where
        I: Iterator<Item = Self>,
    {
        // There may be a more clever way to do this, in a streaming fashion. But for now, I'm
        // just going to do the brute-force approach.
        // let itemsRef = &items;
        let mut acc: HashMap<String, f64> = HashMap::new();
        for word_list in items {
            for word_freq in word_list.words() {
                let entry = acc.entry(word_freq.word.clone());
                *entry.or_insert(0.0) += word_freq.freq;
            }
        }
        // We could be more efficient with this: rather than coming up with the full list, and
        // then trimming it, we could inert-and-trim as we go. Not important for now, though.
        let mut acc_vec: Vec<WordFreq> = acc.into_iter()
            .map(|(word, freq)| WordFreq { word, freq })
            .collect();
        acc_vec.sort_by(|first, second|{
            match second.freq.total_cmp(&first.freq) {
                Ordering::Equal => first.word.cmp(&second.word),
                ne => ne,
            }
        });
        acc_vec.truncate(limit);
        // We could compact now, but probably not worth the CPU. It'll get compacted next time
        // it's filtered, anyway.

        Reified {
            words: Rc::new(acc_vec),
        }
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

    pub fn all_chars(&self) -> FlatMap<Iter, Chars<'_>, fn(&WordFreq) -> Chars<'_>> {
        self.words().flat_map(|w| w.word.chars())
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
