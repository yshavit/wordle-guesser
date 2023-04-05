use pancurses::{endwin, initscr, Input};

use wordlehelper::guesses_ui::GuessGrid;
use wordlehelper::knowledge::GridKnowledge;
use wordlehelper::window_helper;
use wordlehelper::word_list::WordList;

fn main() {
    let window = initscr();
    window_helper::init(&window);

    let mut guess_grid = GuessGrid::<3, 2>::new();
    guess_grid.draw(&window);

    window.refresh();
    loop {
        match window.getch() {
            Some(Input::Character(c)) if c == '\t' => break,
            Some(input) => guess_grid.handle_input(&window, input),
            _ => {}
        }
        guess_grid.draw(&window);
        window.refresh();
    }
    endwin();

    let knowledge = GridKnowledge::from_grid(&guess_grid);
    let mut word_list = WordList::<3>::get_embedded();

    word_list.filter(&knowledge);

    word_list.print(10);
}
