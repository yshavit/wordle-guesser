use crate::ui::widget::Widget;
use pancurses::{Input, Window};
use std::borrow::Cow;
use std::cmp::min;

pub struct TextScroll {
    window: Window,
    title: Option<String>,
    texts: Vec<String>,
    first_visible_idx: usize,
}

impl TextScroll {
    pub fn new(owner: &Window, lines: i32, cols: i32, pos_y: i32, pos_x: i32) -> Self {
        let (owner_max_y, owner_max_x) = owner.get_max_yx();
        let lines_trunc = min(lines, owner_max_y - pos_y);
        let cols_trunc = min(cols, owner_max_x - pos_x);
        let text_scroll = TextScroll {
            window: owner
                .subwin(lines_trunc, cols_trunc, pos_y, pos_x)
                .expect("couldn't create text scroll pane"),
            title: None,
            texts: Vec::new(),
            first_visible_idx: 0,
        };
        text_scroll.redraw();
        text_scroll
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn set_texts(&mut self, texts: Vec<String>) {
        self.texts = texts;
        self.first_visible_idx = 0;
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
}

impl Widget for TextScroll {
    fn title(&self) -> Option<String> {
        (&self.title).as_ref().map(|s| s.to_string())
    }

    fn set_active(&mut self, _active: bool) {
        // nothing
    }

    fn handle_input(&mut self, input: Input) -> Option<Input> {
        match input {
            Input::Character('\x04') => self.scroll_down(), // ctrl-d
            Input::Character('\x15') => self.scroll_up(),   // ctrl-u
            _ => {
                return Some(input);
            }
        }
        None
    }
}

impl TextScroll {
    fn redraw(&self) {
        let (max_y, max_x) = self.window.get_max_yx();
        if max_y < 3 || max_x < 4 {
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
        let main_pane_width = max_x - 4; // 1 for the left │, 3 for │<space>│ of the scroll bar
        let main_pane_width_usize = main_pane_width as usize;

        fn rep_str(ch: char, size: usize) -> String {
            std::iter::repeat(ch).take(size).collect()
        }

        let main_pane_h_bar: String = rep_str('─', main_pane_width_usize);

        let use_title = if max_y >= 5 { &self.title } else { &None };
        match use_title {
            None => {
                self.window.mvaddstr(0, 0, "┌");
                self.window.addstr(&main_pane_h_bar);
                self.window.addstr("┬─┐");
            }
            Some(title) => {
                let title_width = (max_x - 2) as usize;
                let title_truncated = if title.len() <= title_width {
                    Cow::Borrowed(title)
                } else {
                    let mut truncated = title[0..title_width].to_string();
                    truncated.replace_range(title_width - 1..title_width, "…");
                    Cow::Owned(truncated)
                };
                self.window.mvaddstr(0, 0, "╭");
                self.window.addstr(&main_pane_h_bar);
                self.window.addstr("──");
                self.window.addstr("╮");
                // main_pain_width is the total width minus 4. We don't lose any space from the
                // scroll bar for the title, but we still want total width minus 4: 1 on each side
                // for the vertical bars, and then 1 each on each side for padding.

                self.window
                    .mvaddstr(1, 0, format!("│{:<title_width$}│", title_truncated));
                self.window.mvaddstr(2, 0, "┝");
                self.window.addstr(rep_str('━', main_pane_width_usize));
                self.window.addstr("┯━┥");
            }
        };
        let first_body_row = self.window.get_cur_y();

        // Scroll bar
        let num_rows = (max_y - first_body_row - 1) as usize; // -1 for footer
        let num_rows_f64 = num_rows as f64;
        let height = if num_rows >= self.texts.len() {
            0
        } else {
            // "height as float, height as int"
            let height_f = num_rows_f64 / (self.texts.len() as f64) * num_rows_f64;
            let height_i = height_f.round() as usize;
            if height_i >= num_rows {
                height_i - 2 // -2 so that we always have room for the first and last
            } else if height_i == 0 {
                1
            } else {
                height_i
            }
        };
        let scroller_block = if height == 0 {
            None
        } else {
            let num_texts_not_visible = (self.texts.len() - num_rows) as f64;
            // how much "wiggle room" the scroll bar has; that is, how many blocks are not scroller
            let num_scroll_bar_wiggle = num_rows - height;
            let pos_y_ratio = (self.first_visible_idx as f64) / num_texts_not_visible;
            let mut pos_y = (pos_y_ratio * (num_scroll_bar_wiggle as f64)).round() as usize;
            // now, adjust
            if pos_y == 0 && self.first_visible_idx > 0 {
                pos_y = 1;
            } else {
                let last_visible_idx = self.first_visible_idx + num_rows;
                let have_more_texts = last_visible_idx < self.texts.len() - 1;
                let scroller_at_bottom = pos_y == num_scroll_bar_wiggle;
                if have_more_texts && scroller_at_bottom {
                    pos_y -= 1;
                }
            }
            Some(pos_y..(pos_y + height + 1)) // +1 because exclusive
        };

        // text rows
        let empty_str = &("".to_string());
        for main_pane_y in 0..max_y - first_body_row {
            let main_pane_y_usize = main_pane_y as usize;
            let print_scroller = scroller_block
                .as_ref()
                .map(|r| r.contains(&main_pane_y_usize))
                .unwrap_or(false);
            let scroller = if print_scroller { '█' } else { ' ' };
            let text = self
                .texts
                .get(self.first_visible_idx + main_pane_y_usize)
                .unwrap_or(empty_str);
            let write = format!("│{:<main_pane_width_usize$}│{}│", text, scroller);
            self.window.mvaddstr(main_pane_y + first_body_row, 0, write);
        }
        // footer
        self.window.mvaddstr(max_y - 1, 0, "└");
        self.window.addstr(&main_pane_h_bar);
        self.window.addstr("┴─┘");
    }
}

impl Drop for TextScroll {
    fn drop(&mut self) {
        self.window.delch();
    }
}
