use crossterm::event::KeyEvent;
use ratatui::widgets::Widget;

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        None
    }
}

impl Widget for &State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        todo!()
    }
}
