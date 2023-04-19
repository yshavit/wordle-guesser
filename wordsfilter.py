#!/usr/bin/python3
'''
Reads stdin, and assumes the lines are all of form <word><tab><frequency>

It then outputs all the lines within that which (a) are 5 letters, and (b) do NOT have a corresponding 4-letter word plus an "s".

For example, if the file contained:

    three	789
    four	456
    fours	123

... then the output would be:

    three	123

The "four 456" line is omitted because it doesn't have 5 chars, and the "fours 123" line is omitted because "fours" == "four" + "s".
'''

import sys

word_5char_freqs = {}
words_4char_with_s = set()
words_6char_with_s = {}
words_7char_with_es = {}
words_8char_with_ies = {}


def add_to(m, k, v):
    m[k] = m.get(k, 0) + float(v)


def find_plural(m, word):
    result = m.get(word, 0)
    # if result:
    #     print(f'found plural: {word}', file=sys.stderr)
    return result



for line_no, line in enumerate(sys.stdin):
    line = line.strip()
    splits = line.split("\t")
    if len(splits) != 2:
        raise Exception(f'invalid entry on line {line_no}: {line}')
    word, freq = splits
    if not word.isalpha():
        continue
    word_len = len(word)
    if word_len == 5:
        add_to(word_5char_freqs, word, freq)
    elif word_len == 4:
        words_4char_with_s.add(word + 's')
    elif word_len == 6 and word.endswith('s'):
        add_to(words_6char_with_s, word, freq)
    elif word_len == 7 and word.endswith('es'):
        add_to(words_7char_with_es, word, freq)

# now copy them over to a [(word, freq)], editing as you go
out_pairs = []
for word, freq in word_5char_freqs.items():
    if word in words_4char_with_s:
        continue
    freq += find_plural(words_6char_with_s, word + 's')
    freq += find_plural(words_7char_with_es, word + 'es')
    if word.endswith('y'):
        freq += find_plural(words_8char_with_ies, word[:-1] + 'ies')
    out_pairs.append((word, freq))

out_pairs.sort(key=lambda x: x[1], reverse=True)
for word, freq in out_pairs:
    print(f'{word}\t{freq}')


