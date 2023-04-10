use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use wordlehelper::analyze::analyzer;
use wordlehelper::guess::guesses::GuessGrid;
use wordlehelper::guess::known_word_constraints::{CharKnowledge, KnownWordConstraints};
use wordlehelper::word_list::WordList;

fn bench_analyzers(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyzers");
    let words_5c: WordList<5> = WordList::get_embedded(10_000);
    for analyzer in analyzer::standard_suite() {
        group.bench_with_input(
            BenchmarkId::new("filter", analyzer.name()),
            &words_5c,
            |b, words| b.iter(|| analyzer.analyze(words)),
        );
    }
}

fn bench_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");

    for size in [10, 100, 1_000, 2_000, 5_000, 10_000] {
        group.throughput(Throughput::Elements(size));
        let words_5c: WordList<5> = WordList::get_embedded(size as usize);
        let mut grid = GuessGrid::<5, 1>::new();
        let row_knowledge = [
            ('a', CharKnowledge::WrongPosition),
            ('b', CharKnowledge::Missing),
            ('o', CharKnowledge::Missing),
            ('u', CharKnowledge::WrongPosition),
            ('t', CharKnowledge::Missing),
        ];
        let row = grid.guess_mut(0);
        for (ch_idx, (ch, known)) in row_knowledge.into_iter().enumerate() {
            let ch_guess = row.guess_mut(ch_idx);
            ch_guess.set_ch(ch);
            ch_guess.set_knowledge(known);
        }
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(words_5c, &grid),
            |b, (words, grid)| {
                b.iter(|| {
                    words
                        .filter_preview(&KnownWordConstraints::from_grid(grid))
                        .words()
                        .count()
                })
            },
        );
    }
}

criterion_group!(benches, bench_analyzers, bench_filter);
criterion_main!(benches);
