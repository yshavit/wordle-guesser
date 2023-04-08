use pancurses::Window;
use std::cmp::min;

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
                .expect("couldn't create text scroll pane"),
            texts: Vec::new(),
            first_visible_idx: 0,
        }
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

        // Scroll bar
        let num_rows = (max_y - 2) as usize; // -2 for header and footer
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
