use crate::knowledge::GridKnowledge;

pub struct WordFreq {
    pub word: String,
    pub freq: u32,
}

pub struct WordList<const N: usize> {
    words: Vec<WordFreq>,
}

impl<const N: usize> WordList<N> {
    pub fn get_embedded() -> Self {
        let file = include_str!("enwiki-2022-08-29-10K.txt");
        let mut result = WordList {
            words: Vec::with_capacity(file.chars().filter(|c| c == &'\n').count()),
        };
        for line in file.split("\n") {
            let Some((word, freq_str)) = line.split_once(" ") else {
                continue;
            };
            if word.len() != N {
                continue;
            }
            let Ok(freq) = freq_str.parse::<u32>() else {
                continue;
            };
            result.words.push(WordFreq {
                word: word.to_string(),
                freq,
            })
        }
        return result;
    }

    pub fn filter(&mut self, knowledge: &GridKnowledge<N>) {
        self.words
            .retain(|word| knowledge.is_word_possible(&word.word))
    }

    pub fn print(&self, max: usize) {
        for word in self.words.iter().take(max) {
            println!("{}\t({})", word.word, word.freq)
        }
    }
}
