use pancurses::{cbreak, curs_set, endwin, init_pair, initscr, Input, noecho, start_color};
use strum::EnumCount;
use wordlehelper::guesses::{GuessGrid, GuessKnowledge};

fn main() {
    let window = initscr();
    window.keypad(true);
    curs_set(0);
    noecho();
    cbreak();
    start_color();

    for i in 0..GuessKnowledge::COUNT {
        let e = GuessKnowledge::from_repr(i).expect("out of bounds");
        let fg = match e {
            GuessKnowledge::Unknown => pancurses::COLOR_WHITE,
            GuessKnowledge::WrongPosition => pancurses::COLOR_YELLOW,
            GuessKnowledge::Correct => pancurses::COLOR_GREEN,
            GuessKnowledge::Missing => pancurses::COLOR_RED,
        };
        init_pair(e.as_i16(), fg, pancurses::COLOR_BLACK);
    }

    let mut guess_grid = GuessGrid::new();
    guess_grid.draw(&window);
    // let mut guesses = GuessStr::new(5);
    // guesses.draw(&window);

    window.refresh();
    loop {
        match window.getch() {
            Some(Input::KeyAbort) => break,
            Some(input) => guess_grid.handle_input(input),
            _ => {}
        }
        guess_grid.draw(&window);
        window.refresh();
    }
    endwin();
}
