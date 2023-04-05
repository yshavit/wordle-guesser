use strum::{EnumCount, FromRepr};
use crate::window_helper::Color;

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
