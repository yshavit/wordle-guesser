use pancurses::{cbreak, curs_set, endwin, init_pair, initscr, Input, noecho, start_color};
use strum::EnumCount;
use wordlehelper::guesses::{GuessKnowledge, GuessStr};

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

    let mut guesses = GuessStr::new(5);
    guesses.draw(&window);

    window.refresh();
    loop {
        match window.getch() {
            Some(Input::KeyUp) => guesses.cycle_guess_knowledge(true),
            Some(Input::KeyDown) => guesses.cycle_guess_knowledge(false),
            Some(Input::KeyRight) => guesses.move_active(true),
            Some(Input::KeyLeft) => guesses.move_active(false),
            Some(Input::Character(c)) => match c {
                '\x7F' => guesses.unset_ch(),
                '\n' => { /* TODO handle newline */ },
                _ => guesses.set_ch(c)
            }
            Some(Input::KeyAbort) => break,
            _ => {}
        }
        guesses.draw(&window);
        window.refresh();
    }
    endwin();
}
