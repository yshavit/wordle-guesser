use pancurses::{chtype, curs_set, init_pair, noecho, raw, start_color, Window};
use std::cmp::{min};

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
        return WindowState {
            orig_y,
            orig_x,
            window,
            orig_attrs,
            orig_color,
        };
    }

    pub fn set_color(&self, color: Color) {
        self.window.color_set(color as i16);
    }
}

pub fn init(window: &Window) {
    window.keypad(true);
    curs_set(0);
    noecho();
    raw();
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

pub struct TextScroll {
    window: Window,
    texts: Vec<String>,
    first_visible_idx: usize,
}

impl TextScroll {
    pub fn new(owner: &Window, lines: i32, cols: i32, pos_y: i32, pos_x: i32) -> Self {
        let (owner_max_y, owner_max_x) = owner.get_max_yx();
        let lines_trunc = min(lines, owner_max_y - pos_y);
        let cols_trunc = min(cols, owner_max_x - pos_x);
        TextScroll {
            window: owner
                .subwin(lines_trunc, cols_trunc, pos_y, pos_x)
                .expect("couldn't create subwindow"),
            texts: Vec::new(),
            first_visible_idx: 0,
        }
    }

    pub fn set_texts(&mut self, texts: Vec<String>) {
        self.texts = texts;
        self.redraw();
    }

    pub fn scroll_down(&mut self) {
        let new_last_visible = self.first_visible_idx + (self.window.get_max_y() as usize) + 1;
        if new_last_visible <= self.texts.len() {
            self.first_visible_idx += 1;
        }
        self.redraw();
    }

    pub fn scroll_up(&mut self) {
        self.first_visible_idx = self.first_visible_idx.saturating_sub(1);
        self.redraw();
    }

    fn redraw(&self) {
        let (max_y, max_x) = self.window.get_max_yx();
        if max_y < 2 || max_x < 4 {
            return;
        }
        // We're going for something like this:
        // ┌──────┬─┐
        // │hello │█│
        // │there │ │
        // │pooh  │ │
        // │bear  │ │
        // └──────┴─┘

        // header
        let main_pane_width = max_x - 4;
        let main_pane_width_usize = main_pane_width as usize;

        let main_pane_h_bar: String = std::iter::repeat('─').take(main_pane_width_usize).collect();

        self.window.mvaddstr(0, 0, "┌");
        self.window.printw(&main_pane_h_bar);
        self.window.printw("┬─┐");

        // scroller block has a few simple rules:
        // 1. First we get its height:
        //    a. If we have at least as many rows as texts, the height is 0.
        //    b. Else, it's (num_rows / num_texts * num_rows).
        //    c. Then we round it, and if that's == num_rows, we subtract 2 (so that it never looks
        //       full if we don't see all the elements).
        // 2. Then, we get its position:
        //    a. Initially it's just (first_visible_idx / number_of_texts), rounded.
        //    b. If the first element isn't visible, then the position is at least 1.
        //    c. If the last element isn't visible, then the position is such that there's at least
        //       one empty row at the bottom.
        //

        let num_rows = (max_y - 2) as usize; // -2 for header and footer
        let num_rows_f64 = num_rows as f64;

        let height = if num_rows >= self.texts.len() {
            0
        } else {
            // "height as float, height as int"
            let height_f = num_rows_f64 / (self.texts.len() as f64) * num_rows_f64;
            let height_i = height_f.round() as usize;
            if height_i >= num_rows {
                height_i - 2
            } else if height_i == 0 {
                1
            } else {
                height_i
            }
        };
        let pos_y = {
            0 as usize
            // // "position y as float, position y as int"
            // let pos_y_wiggle = num_rows_f64 - (height as f64);
            // let pos_y_ratio = (self.first_visible_idx as f64) / ((self.texts.len() - num_rows) as f64);
            // let pos_y_f = pos_y_ratio * pos_y_wiggle;
            // let pos_y_i = pos_y_f.round() as usize;
            // if pos_y_i == 0 && self.first_visible_idx > 0 {
            //     1
            // } else {
            //     let last_visible_idx = self.first_visible_idx + num_rows - 1;
            //     let last_text_not_visible = last_visible_idx < (self.texts.len() - 1);
            //     let scroll_hit_bottom = pos_y_i + height >= num_rows - 1;
            //     if last_text_not_visible && scroll_hit_bottom {
            //         pos_y_i - 1
            //     } else {
            //         pos_y_i
            //     }
            // }
        };

        let scroller_block = if height == 0 {
            None
        } else {
            Some(pos_y..(pos_y + height))
        };

        // text rows
        let empty_str = &("".to_string());
        for pos_y in 1..max_y - 1 {
            let pos_within_main = (pos_y - 1) as usize; // header row doesn't count
            let print_scroller = scroller_block
                .as_ref()
                .map(|r| r.contains(&pos_within_main))
                .unwrap_or(false);
            let scroller = if print_scroller { '█' } else { ' ' };
            let text = self
                .texts
                .get(self.first_visible_idx + pos_within_main)
                .unwrap_or(empty_str);
            let write = format!("│{:<main_pane_width_usize$}│{}│", text, scroller);
            self.window.mvaddstr(pos_y, 0, write);
        }
        // footer
        self.window.mvaddstr(max_y - 1, 0, "└");
        self.window.printw(&main_pane_h_bar);
        self.window.printw("┴─┘");
    }
}

impl Drop for TextScroll {
    fn drop(&mut self) {
        self.window.delch();
    }
}
