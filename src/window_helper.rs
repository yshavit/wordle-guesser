use pancurses::{cbreak, chtype, curs_set, init_pair, noecho, start_color, Window};
use strum::{EnumCount, FromRepr};

#[derive(Copy, Clone, PartialEq, EnumCount, FromRepr)]
pub enum Color {
    StandardForeground,
    Warning,
    Good,
    Error,
    Hidden,
}

pub struct WindowState<'a> {
    pub orig_y: i32,
    pub orig_x: i32,
    window: &'a Window,
    orig_attrs: chtype,
    orig_color: i16,
}

impl<'a> Drop for WindowState<'a> {
    fn drop(&mut self) {
        self.window.attrset(self.orig_attrs);
        self.window.color_set(self.orig_color);
    }
}

impl<'a> WindowState<'a> {
    pub fn new(window: &Window) -> WindowState {
        let (orig_y, orig_x) = window.get_cur_yx();
        let (orig_attrs, orig_color) = window.attrget();
        return WindowState{ orig_y, orig_x, window, orig_attrs, orig_color }
    }

    pub fn set_color(&self, color: Color) {
        self.window.color_set(color as i16);
    }
}

pub fn init(window: &Window) {
    window.keypad(true);
    curs_set(0);
    noecho();
    cbreak();
    start_color();

    for i in 0..Color::COUNT {
        let e = Color::from_repr(i).expect("out of bounds");
        let fg = match e {
            Color::StandardForeground => pancurses::COLOR_WHITE,
            Color::Warning => pancurses::COLOR_YELLOW,
            Color::Good => pancurses::COLOR_GREEN,
            Color::Error => pancurses::COLOR_RED,
            Color::Hidden => pancurses::COLOR_BLACK,
        };
        init_pair(e as i16, fg, pancurses::COLOR_BLACK);
    }
}
