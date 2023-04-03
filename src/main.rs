use pancurses::{cbreak, curs_set, endwin, init_pair, initscr, noecho, start_color, Input, Window};
use std::cmp::min;
use strum::{EnumCount, FromRepr};

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
            GuessKnowledge::Missing => pancurses::COLOR_RED,
            GuessKnowledge::WrongPosition => pancurses::COLOR_YELLOW,
            GuessKnowledge::Correct => pancurses::COLOR_GREEN,
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
            Some(Input::Character(c)) => guesses.set_ch(c),
            Some(Input::KeyAbort) => break,
            _ => {}
        }
        guesses.draw(&window);
        window.refresh();
    }
    endwin();
}

struct BoxStyle<'a> {
    top: &'a str,
    vert: char,
    bot: &'a str,
}

const STYLE_ACTIVE: BoxStyle = BoxStyle {
    top: "┏━┓",
    vert: '┃',
    bot: "┗━┛",
};

const STYLE_INACTIVE: BoxStyle = BoxStyle {
    top: "╭─╮",
    vert: '│',
    bot: "╰─╯",
};

#[derive(Copy, Clone, EnumCount, FromRepr)]
enum GuessKnowledge {
    Unknown,
    Missing,
    WrongPosition,
    Correct,
}

impl GuessKnowledge {
    fn as_i16(&self) -> i16 {
        (*self as usize) as i16
    }
}

struct GuessChar {
    knowledge: GuessKnowledge,
    ch: Option<char>,
}

impl GuessChar {
    fn draw(&self, window: &Window, style: &BoxStyle) {
        let (orig_y, orig_x) = window.get_cur_yx();
        let ch = self.ch.unwrap_or(' ');

        let (orig_attrs, orig_color) = window.attrget();
        window.color_set(self.knowledge.as_i16());

        _ = window.printw(style.top);
        _ = window.mvprintw(
            orig_y + 1,
            orig_x,
            format!("{}{}{}", style.vert, ch, style.vert),
        );
        _ = window.mvprintw(orig_y + 2, orig_x, style.bot);

        window.attrset(orig_attrs);
        window.color_set(orig_color);
    }

    fn set_knowledge(&mut self, knowledge: GuessKnowledge) {
        self.knowledge = knowledge;
    }

    fn set_ch(&mut self, ch: Option<char>) {
        self.ch = ch;
    }
}

struct GuessStr {
    guesses: Vec<GuessChar>,
    active: Option<usize>,
}

impl GuessStr {
    fn new(size: usize) -> Self {
        let mut result = GuessStr {
            guesses: Vec::with_capacity(size),
            active: if size > 0 { Some(0) } else { None },
        };
        for _ in 0..size {
            result.guesses.push(GuessChar {
                knowledge: GuessKnowledge::Unknown,
                ch: None,
            })
        }
        return result;
    }

    fn draw(&self, window: &Window) {
        let (orig_y, orig_x) = window.get_cur_yx();
        for (i, guess) in self.guesses.iter().enumerate() {
            window.mv(orig_y, orig_x + (i as i32 * 4));
            let style = if self.active.map(|active| active == i).unwrap_or(false) {
                STYLE_ACTIVE
            } else {
                STYLE_INACTIVE
            };
            guess.draw(window, &style);
        }
        window.mv(orig_y, orig_x);
    }

    fn cycle_guess_knowledge(&mut self, up: bool) {
        if let Some(active) = self.active {
            let guess = self.guesses.get_mut(active).expect("out of bounds");
            let curr_knowledge = guess.knowledge;
            let next_idx = incr_usize(curr_knowledge as usize, GuessKnowledge::COUNT, up, true);
            let next = GuessKnowledge::from_repr(next_idx)
                .expect(&format!("out of range for {}", next_idx));
            guess.set_knowledge(next);
        }
    }

    fn move_active(&mut self, right: bool) {
        if self.guesses.is_empty() {
            return;
        }
        match self.active {
            None => {
                if right {
                    self.active = Some(self.guesses.len() - 1)
                } else {
                    self.active = Some(0)
                }
            }
            Some(curr) => {
                self.active = Some(incr_usize(curr, self.guesses.len(), right, false));
            }
        }
    }

    fn set_ch(&mut self, ch: char) {
        return if ch == '\x7F' {
            // delete
            if let Some(idx) = self.active {
                let active = self.guesses.get_mut(idx).expect("out of bounds");
                if let None = active.ch {
                    self.move_active(false);
                }
                self.set_ch_direct(None)
            }
        } else if ch.is_ascii_alphabetic() {
            self.set_ch_direct(Some(ch.to_ascii_uppercase()));
            self.move_active(true);
        };
    }

    fn set_ch_direct(&mut self, ch: Option<char>) {
        if let Some(idx) = self.active {
            let active = self.guesses.get_mut(idx).expect("out of bounds");
            active.set_ch(ch);
        }
    }
}

fn incr_usize(u: usize, max_exclusive: usize, up: bool, wrap: bool) -> usize {
    match u.checked_add_signed(if up { 1 } else { -1 }) {
        Some(res) => {
            if wrap {
                res % max_exclusive
            } else {
                min(res, max_exclusive - 1)
            }
        }
        None => {
            if wrap {
                max_exclusive - 1
            } else {
                0
            }
        }
    }
}
