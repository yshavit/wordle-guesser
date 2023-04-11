use crate::analyze::analyzer;
use crate::ui::analyzers_ui::AnalyzersUI;
use crate::ui::guesses_ui::GuessesUI;
use crate::ui::text_scroll_pane::TextScroll;
use crate::ui::widget::Widget;
use crate::ui::window_helper::init;

use pancurses::{endwin, Input, Window};

pub struct MainWindow<const N: usize, const R: usize> {
    window: Window,
}

impl<const N: usize, const R: usize> Drop for MainWindow<N, R> {
    fn drop(&mut self) {
        endwin();
    }
}

impl<const N: usize, const R: usize> MainWindow<N, R> {
    pub fn init() -> Self {
        MainWindow { window: init() }
    }

    pub fn run_main_loop(&mut self) {
        let mut guesses_ui: GuessesUI<N, R> = GuessesUI::new(&self.window, 0, 0);

        let mut analyzers_ui = AnalyzersUI::new(
            self.create_text_scroll(None, 30, 0, 34),
            analyzer::standard_suite(),
        );

        loop {
            guesses_ui.handle_new_knowledge(|possible_words| analyzers_ui.analyze(possible_words));

            self.refresh();

            let mut maybe_input = self.get_input();
            let mut widgets: Vec<&mut dyn Widget> = vec![(&mut guesses_ui), (&mut analyzers_ui)];

            if let Some(input) = maybe_input {
                match input {
                    Input::Character(c) if c == '\x03' => {
                        // ctrl-c
                        return;
                    }
                    _ => {
                        for widget in widgets.iter_mut() {
                            maybe_input = widget.handle_input(input);
                            if maybe_input == None {
                                break;
                            }
                        }
                    }
                };

            }
        }
    }

    pub fn refresh(&self) {
        self.window.touch();
        self.window.refresh();
    }

    pub fn get_input(&self) -> Option<Input> {
        self.window.getch()
    }

    pub fn create_text_scroll(
        &self,
        lines: Option<i32>,
        cols: i32,
        pos_y: i32,
        pos_x: i32,
    ) -> TextScroll {
        TextScroll::new(
            &self.window,
            lines.unwrap_or(self.window.get_max_y()),
            cols,
            pos_y,
            pos_x,
        )
    }
}
