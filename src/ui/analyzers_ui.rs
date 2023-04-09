use crate::analyze::analyzer::{Analyzer, ScoredWord};
use crate::ui::text_scroll_pane::TextScroll;
use crate::ui::widget::Widget;
use crate::util::{incr_usize, WRAP};
use crate::word_list::WordList;
use pancurses::Input;

pub struct AnalyzersUI<const N: usize> {
    output: TextScroll,
    analyzers: Vec<Analyzer<N>>,
    active_analyzer: usize,
}

impl<const N: usize> AnalyzersUI<N> {
    pub fn new(output: TextScroll, analyzers: Vec<Analyzer<N>>) -> Self {
        AnalyzersUI {
            output,
            analyzers,
            active_analyzer: 0,
        }
    }

    pub fn analyze(&mut self, word_list: &WordList<N>) {
        let Some(analyzer) = self.analyzers.get(self.active_analyzer) else {
            return
        };
        self.output.set_title(&analyzer.name);
        let mut scored = (analyzer.func)(word_list);
        scored.sort();
        ScoredWord::normalize_scores(&mut scored);
        let texts: Vec<String> = scored
            .iter()
            .map(|sw| format!("{}: {:.3}", sw.word, sw.score))
            .collect();
        self.output.set_texts(texts);
    }
}

impl<const N: usize> Widget for AnalyzersUI<N> {
    fn title(&self) -> Option<&str> {
        self.analyzers
            .get(self.active_analyzer)
            .map(|a| &a.name as &str)
    }

    fn set_active(&mut self, _active: bool) {
        // nothing
    }

    fn handle_input(&mut self, input: Input) -> Option<Input> {
        match input {
            Input::Character('\t') => {
                incr_usize(&mut self.active_analyzer, self.analyzers.len(), true, WRAP);
                // TODO how do I redraw??
                None
            }
            _ => self.output.handle_input(input),
        }
    }
}
