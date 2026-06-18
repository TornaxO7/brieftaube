use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct State {}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) {}
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Widget::render(
            Paragraph::new("This is an example mail").block(Block::bordered().title("Content")),
            area,
            buf,
        )
    }
}
