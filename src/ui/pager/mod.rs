use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame, widgets::Widget};

#[derive(Debug, Default)]
pub struct State {}

impl Widget for &State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        todo!()
    }
}
