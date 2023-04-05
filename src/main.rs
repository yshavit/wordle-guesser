use std::time::Instant;
use pancurses::{endwin, initscr, Input};

use wordlehelper::guesses_ui::GuessGrid;
use wordlehelper::knowledge::GridKnowledge;
use wordlehelper::window_helper;
use wordlehelper::word_list::WordList;

fn main() {
    // let window = initscr();
    // window_helper::init(&window);
    //
    // let mut guess_grid = GuessGrid::<5, 6>::new();
    // guess_grid.draw(&window);
    //
    // window.refresh();
    // loop {
    //     match window.getch() {
    //         Some(Input::Character(c)) if c == '\t' => break,
    //         Some(input) => guess_grid.handle_input(&window, input),
    //         _ => {}
    //     }
    //     guess_grid.draw(&window);
    //     window.refresh();
    // }
    // endwin();

    let guess_grid = GuessGrid::generate_dummy_data();

    let knowledge = GridKnowledge::from_grid(&guess_grid);
    println!("one");
    let read_start = Instant::now();
    let mut word_list = WordList::<5>::get_embedded(6000);
    let elapsed = read_start.elapsed();
    println!("read words in {} ms", elapsed.as_millis());

    word_list.filter(&knowledge);

    word_list.print();
}
