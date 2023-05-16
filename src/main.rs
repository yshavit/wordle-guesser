use std::env;
use wordlehelper::analyze::analyzer;
use wordlehelper::analyze::auto_guesser::AutoGuesser;
use wordlehelper::guess::known_word_constraints::CharKnowledge;
use wordlehelper::ui::tui::MainWindow;
use wordlehelper::word_list::WordList;

fn main() {
    let try_words: Vec<String> = env::args().skip(1).collect();
    if try_words.is_empty() {
        let mut main_window: MainWindow<5, 6> = MainWindow::init();
        main_window.run_main_loop();
    } else {
        let count = try_words.len();
        let auto_guesser: AutoGuesser<5, 6> = AutoGuesser {
            answer_words: try_words,
            words_list: WordList::get_embedded_std(),
            analyzers: analyzer::standard_suite(),
        };
        for result in auto_guesser.guess_all() {
            if count > 0 {
                println!("{}:", result.answer);
            }
            for analyzer_result in result.analyzer_results {
                println!(
                    "    {}: {} in {}",
                    analyzer_result.name,
                    analyzer_result.result,
                    analyzer_result.guesses.len()
                );
                for row in analyzer_result.guesses {
                    print!("      ");
                    for guess_ch in row.chars() {
                        let display = match guess_ch.knowledge() {
                            CharKnowledge::Unknown => "⤵️",
                            CharKnowledge::WrongPosition => "🟨",
                            CharKnowledge::Correct => "🟩",
                            CharKnowledge::Missing => "⬛️",
                        };
                        print!("{}", display)
                    }
                    println!();
                }
                println!();
            }
        }
    }
}
