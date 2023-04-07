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

word_5char_freqs = []
words_4char_with_s = set()

for line in sys.stdin:
    line = line.strip()
    word, freq = line.split("\t")
    if len(word) == 5:
        word_5char_freqs.append((word, freq))
    elif len(word) == 4:
        words_4char_with_s.add(word + 's')

for word, freq in word_5char_freqs:
    if word not in words_4char_with_s:
        print(f'{word}\t{freq}')


