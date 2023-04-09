use crate::analyze::char_stats::CharCounts;
use crate::analyze::scored_chars::ScoredChars;

use crate::ui::guesses_ui::GuessesUI;
use crate::ui::text_scroll_pane::TextScroll;
use crate::ui::widget::Widget;
use crate::ui::window_helper::init;
use crate::word_list::WordList;
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

        let mut words_window = self.create_text_scroll(None, 30, 0, 28);
        let mut scores_window = self.create_text_scroll(None, 30, 0, 64);

        words_window.set_title(Some(
            "Words whatever 1234567890abcdefghijklmnopqrstuvwzyz".to_string(),
        ));
        let refresh_words_list = true;

        loop {
            if refresh_words_list {
                let known_constraints = guesses_ui.get_known_constraints();
                // TODO we can keep just one list on the outside, and whittle it time every time the
                // user presses "enter"
                let mut possible_words = WordList::<N>::get_embedded(10000);
                possible_words.filter(&known_constraints);

                let char_counts = CharCounts::new(&possible_words);
                let scores = ScoredChars::new(&possible_words, &char_counts);
                scores_window.set_texts(
                    scores
                        .all_word_scores()
                        .iter()
                        .take(50)
                        .map(|(word, score)| format!("{}: {:.3}", word, score))
                        .collect(),
                );

                words_window.set_texts(
                    possible_words
                        .words()
                        .iter()
                        .map(|wf| wf.word.to_string())
                        .collect(),
                );
            }

            self.refresh();

            let mut maybe_input = self.get_input();
            let mut widgets: Vec<&mut dyn Widget> = vec![
                (&mut guesses_ui),
                (&mut scores_window),
                (&mut words_window),
            ];

            while let Some(input) = maybe_input {
                match input {
                    Input::Character(c) if c == '\x03' => {
                        // ctrl-c
                        return;
                    },
                    _ => {
                        for widget in widgets.iter_mut() {
                            maybe_input = widget.handle_input(input);
                            if maybe_input == None {
                                break
                            }
                        }
                    },
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
