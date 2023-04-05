use crate::guesses_ui::{GuessGrid, GuessStr};
use crate::window_helper::Color;
use std::collections::HashSet;
use strum::{EnumCount, FromRepr};

#[derive(Copy, Clone, PartialEq, EnumCount, FromRepr)]
pub enum CharKnowledge {
    Unknown,
    WrongPosition,
    Correct,
    Missing,
}

impl CharKnowledge {
    pub fn color(&self) -> Color {
        match self {
            CharKnowledge::Unknown => Color::StandardForeground,
            CharKnowledge::WrongPosition => Color::Warning,
            CharKnowledge::Correct => Color::Good,
            CharKnowledge::Missing => Color::Error,
        }
    }
}

pub struct GridKnowledge<const N: usize> {
    fully_known: [Option<char>; N],
    wrong_positions: Vec<HashSet<char>>,
    missing: HashSet<char>,
}

impl<const N: usize> GridKnowledge<N> {
    pub fn from_grid<const R: usize>(grid: &GuessGrid<N, R>) -> Self {
        // initial info
        let mut result = GridKnowledge {
            fully_known: [None; N],
            wrong_positions: Vec::with_capacity(N),
            missing: HashSet::new(),
        };
        for _ in 0..N {
            result.wrong_positions.push(HashSet::new());
        }

        grid.rows().for_each(|r| result.add_row(r));

        return result;
    }

    pub fn add_row(&mut self, str: &GuessStr<N>) {
        for (idx, guess_ch) in str.chars().enumerate() {
            let Some(ch) = guess_ch.ch else {
                continue;
            };
            match guess_ch.knowledge {
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
