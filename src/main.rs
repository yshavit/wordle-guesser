use pancurses::Input;
use wordlehelper::analyze::char_stats::CharCounts;
use wordlehelper::analyze::scored_chars::ScoredChars;

use wordlehelper::guess::guesses::GuessGrid;
use wordlehelper::guess::known_word_constraints::KnownWordConstraints;
use wordlehelper::ui::tui::{MainWindow, UserAction};
use wordlehelper::ui::window_helper;
use wordlehelper::word_list::WordList;

fn main() {
    window_helper::init();

    let mut main_window = MainWindow::init();
    let mut guess_grid = GuessGrid::<5, 6>::new();
    main_window.draw_guess_grid(&guess_grid);

    let mut words_window = main_window.create_text_scroll(None, 30, 0, 28);
    let mut scores_window = main_window.create_text_scroll(None, 30, 0, 64);
    let mut refresh_words_list = true;

    loop {
        if refresh_words_list {
            let known_constraints = KnownWordConstraints::from_grid(&guess_grid);
            // TODO we can keep just one list on the outside, and whittle it time every time the
            // user presses "enter"
            let mut possible_words = WordList::<5>::get_embedded(10000);
            possible_words.filter(&known_constraints);

            let char_counts = CharCounts::new(&possible_words);
            let scores = ScoredChars::new(&possible_words, &char_counts);
            scores_window.set_texts(
                scores
                    .all_word_scores()
                    .iter()
                    .take(50)
                    .map(|(word, score)| format!("{}: {:.3}", word, score))
                    .collect(),
            );

            words_window.set_texts(
                possible_words
                    .words()
                    .iter()
                    .map(|wf| wf.word.to_string())
                    .collect(),
            );
            refresh_words_list = false;
        }

        main_window.draw_guess_grid(&guess_grid);
        main_window.refresh();

        // TODO probably move all of this to MainWindow
        let action = match main_window.get_input() {
            Some(Input::Character(c)) if c == '\x03' => {
                // ctrl-c
                break;
            }
            Some(Input::Character(c)) if c == '\x04' => {
                // ctrl-d
                words_window.scroll_down();
                UserAction::Other
            }
            Some(Input::Character(c)) if c == '\x15' => {
                // ctrl-u
                words_window.scroll_up();
                UserAction::Other
            }
            Some(input) => main_window.handle_input(&mut guess_grid, input),
            _ => UserAction::Other,
        };
        match action {
            UserAction::SubmittedRow | UserAction::ChangedKnowledge => refresh_words_list = true,
            _ => {}
        }
    }
}
