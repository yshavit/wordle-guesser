use pancurses::{endwin, initscr, Input};

use wordlehelper::guesses_ui::GuessGrid;
use wordlehelper::window_helper;

fn main() {
    let window = initscr();
    window_helper::init(&window);

    let mut guess_grid = GuessGrid::<3, 2>::new();
    guess_grid.draw(&window);

    window.refresh();
    loop {
        match window.getch() {
            Some(Input::KeyAbort) => break,
            Some(input) => guess_grid.handle_input(&window, input),
            _ => {}
        }
        guess_grid.draw(&window);
        window.refresh();
    }
    endwin();
}
