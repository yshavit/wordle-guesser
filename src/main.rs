use std::env;
use wordlehelper::analyze::analyzer;
use wordlehelper::analyze::auto_guesser::{AutoGuesser, GuessResult};
use wordlehelper::ui::tui::MainWindow;
use wordlehelper::word_list::WordList;

fn main() {
    let try_words: Vec<String> = env::args().skip(1).collect();
    if try_words.is_empty() {
        let mut main_window: MainWindow<5, 6> = MainWindow::init();
        main_window.run_main_loop();
    } else {
        let auto_guesser: AutoGuesser<5, 6> = AutoGuesser {
            answer_words: try_words,
            words_list: WordList::get_embedded(10_000),
            analyzers: analyzer::standard_suite(),
        };
        for result in auto_guesser.guess_all() {
            println!("{}:", result.answer);
            for analyzer_result in result.analyzer_results {
                print!("    {}: ", analyzer_result.name);
                match analyzer_result.result {
                    GuessResult::Success(tries) => {
                        println!("Succeeded in {}: {}", tries.len(), tries.join(" "))
                    }
                    GuessResult::Failure => println!("FAILED"),
                }
            }
        }
    }
}
