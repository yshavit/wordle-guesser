use pancurses::{endwin, initscr, Input};

use wordlehelper::guesses_ui::{GuessGrid, UserAction};
use wordlehelper::knowledge::GridKnowledge;
use wordlehelper::window_helper;
use wordlehelper::window_helper::TextScroll;
use wordlehelper::word_list::WordList;

fn main() {
    let window = initscr();
    window_helper::init(&window);

    let mut guess_grid = GuessGrid::<5, 6>::new();
    guess_grid.draw(&window);

    let mut words_window = TextScroll::new(&window, window.get_max_y(), 30, 0, 28);
    let mut refresh_words_list = true;

    loop {
        if refresh_words_list {
            let knowledge = GridKnowledge::from_grid(&guess_grid);
            let mut possible_words = WordList::<5>::get_embedded(10000);
            possible_words.filter(&knowledge);
            words_window.set_texts(
                possible_words
                    .words()
                    .iter()
                    .map(|wf| wf.word.to_string())
                    .collect(),
            );
            refresh_words_list = false;
        }

        window.touch();
        guess_grid.draw(&window);
        window.refresh();

        let action = match window.getch() {
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
            Some(input) => guess_grid.handle_input(&window, input),
            _ => UserAction::Other,
        };
        match action {
            UserAction::SubmittedRow | UserAction::ChangedKnowledge => refresh_words_list = true,
            _ => {}
        }
    }
    endwin();
}
