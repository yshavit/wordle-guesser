use crate::known_word_constraints::KnownWordConstraints;

pub struct WordFreq {
    pub word: String,
    pub freq: u32,
}

pub struct WordList<const N: usize> {
    words: Vec<WordFreq>,
}

impl<const N: usize> WordList<N> {
    pub fn get_embedded(limit: usize) -> Self {
        let file = include_str!("words-5chars.txt");
        let mut result = WordList {
            words: Vec::with_capacity(file.chars().filter(|c| c == &'\n').count()),
        };
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
            result.words.push(WordFreq {
                word: word.to_ascii_uppercase(),
                freq,
            });
            if result.words.len() >= limit {
                break;
            }
        }
        return result;
    }

    pub fn filter(&mut self, knowledge: &KnownWordConstraints<N>) {
        self.words
            .retain(|word| knowledge.is_word_possible(&word.word))
    }

    pub fn words(&self) -> &Vec<WordFreq> {
        &self.words
    }

    pub fn print(&self) {
        for word in self.words.iter() {
            println!("{}\t({})", word.word, word.freq)
        }
    }
}
