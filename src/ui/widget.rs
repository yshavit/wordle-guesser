use pancurses::Input;

pub trait Widget {
    fn title(&self) -> Option<String>;
    fn set_active(&mut self, active: bool);
    fn handle_input(&mut self, input: Input) -> Option<Input>;
}
