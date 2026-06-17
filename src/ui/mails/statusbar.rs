use ratatui::{
    symbols,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct State {}

impl Widget for &State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Widget::render(
            Paragraph::new("Statusbar content").block(Block::bordered()),
            area,
            buf,
        )
    }
}
