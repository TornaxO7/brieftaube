use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct Statusbar {}

impl StatefulWidget for Statusbar {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered();
        let header_area = block.inner(area);

        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Fill(0),
            Constraint::Fill(0),
        ])
        .areas(header_area);

        Widget::render(block, area, buf);
        Widget::render(
            Paragraph::new("Left").alignment(HorizontalAlignment::Left),
            left,
            buf,
        );
        Widget::render(
            Paragraph::new(state.screen_name).alignment(HorizontalAlignment::Center),
            center,
            buf,
        );
        Widget::render(
            Paragraph::new("Right").alignment(HorizontalAlignment::Right),
            right,
            buf,
        );
    }
}
