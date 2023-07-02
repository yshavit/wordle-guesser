pub fn uniq_chars(word: &str) -> CharsSet {
    CharsSet {
        counts: chars_count(word.chars()),
    }
}

pub fn chars_count<I>(chars: I) -> CharsCount
where
    I: Iterator<Item = char>,
{
    let mut chars_count = CharsCount::default();
    for ch in chars.into_iter() {
        chars_count.increment(ch);
    }
    chars_count
}

const A_USIZE: usize = 'A' as usize;
const Z_USIZE: usize = 'Z' as usize;
const NUM_CHARS: usize = Z_USIZE - A_USIZE + 1;

#[derive(Default, Copy, Clone)]
pub struct CharsCount {
    counts: [u32; NUM_CHARS],
}

impl CharsCount {
    #[inline]
    pub fn get(&self, ch: char) -> u32 {
        if !ch.is_ascii_alphabetic() {
            return 0;
        }
        let ch = ch.to_ascii_uppercase();
        return self.counts[ch as usize - A_USIZE];
    }

    #[inline]
    pub fn get_mut(&mut self, ch: char) -> Option<&mut u32> {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        let ch = ch.to_ascii_uppercase();
        return Some(&mut self.counts[ch as usize - A_USIZE]);
    }

    #[inline]
    pub fn increment(&mut self, ch: char) {
        if let Some(count) = self.get_mut(ch) {
            *count += 1;
        }
    }

    #[inline]
    pub fn decrement(&mut self, ch: char) {
        if let Some(count) = self.get_mut(ch) {
            *count -= 1;
        }
    }

    #[inline]
    pub fn reset_all(&mut self) {
        self.counts.fill(0);
    }
}

pub struct CharsSet {
    counts: CharsCount,
}

impl CharsSet {
    pub fn contains(&self, ch: char) -> bool {
        self.counts.get(ch) > 0
    }
}

impl IntoIterator for CharsSet {
    type Item = char;
    type IntoIter = CharsIter;

    fn into_iter(self) -> Self::IntoIter {
        CharsIter::new(self.counts)
    }
}

pub struct CharsIter {
    counts: CharsCount,
    next_idx: usize,
}

impl CharsIter {
    fn new(counts: CharsCount) -> Self {
        let mut result = CharsIter {
            counts,
            next_idx: 0,
        };
        result.find_next();
        result
    }

    /// Finds the next index with a nonzero count, which may be the current one.
    fn find_next(&mut self) {
        loop {
            if self.next_idx >= NUM_CHARS {
                break;
            }
            if self.counts.counts[self.next_idx] > 0 {
                break;
            }
            self.next_idx += 1;
        }
    }
}

impl Iterator for CharsIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_idx >= NUM_CHARS {
            return None;
        }
        let result = (self.next_idx + A_USIZE) as u8 as char;
        self.next_idx += 1;
        self.find_next();
        Some(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn uniq_chars_empty() {
        let set = uniq_chars("");

        assert!(!set.contains('a'));
        assert!(!set.contains('A'));

        let mut iter = set.into_iter();
        assert_eq!(None, iter.next());
    }

    #[test]
    fn uniq_chars_some() {
        let set = uniq_chars("ACZ");
        assert!(set.contains('a'));
        assert!(set.contains('A'));
        assert!(set.contains('Z'));

        let mut iter = set.into_iter();

        assert_eq!(Some('A'), iter.next());
        assert_eq!(Some('C'), iter.next());
        assert_eq!(Some('Z'), iter.next());
        assert_eq!(None, iter.next());
    }
}
