# Wordle Helper

Interactive CLI tool for solving Wordle puzzles.

## Developing

Please run the following when you initially check the repo out (if you plan on pushing code):

    git config core.hooksPath .githooks

Building is just the usual `cargo build`, `cargo run`, etc. Nothing fancy here.

## Libraries (other than what's in Cargo.toml)

- `words-5chars.txt` comes from https://norvig.com/ngrams/ ([direct link][1])
- `words-5chars-wiktionary-gutenberg.txt` comes from [Wiktionary's frequency list from Project Gutenberg][2]
- `words-5chars-hermitdave.txt` comes from github.com/hermitdave/FrequencyWords's `en_full.txt` ([direct link][3])  

They have been truncated to 5-char words and filtered to remove plurals using `wordsfilter.py`.


[1]: https://norvig.com/ngrams/count_1w.txt
[2]: https://en.wiktionary.org/wiki/Wiktionary:Frequency_lists/English/Project_Gutenberg
[3]: https://github.com/hermitdave/FrequencyWords/tree/072bbed282316a23651aa7068c7173aa7898cf80/content/2018/en
