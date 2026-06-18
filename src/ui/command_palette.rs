use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{List, ListDirection, ListState, StatefulWidget, Widget},
};
use ratatui_textarea::TextArea;

#[derive(Debug, Default)]
pub struct State {
    input: TextArea<'static>,
}

impl State {
    pub fn handle_event(&mut self, event: KeyEvent) {}
}

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [results, search] = area.layout(
            &Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)]),
        );

        StatefulWidget::render(
            List::new(["Result 1", "Result 2"])
                .highlight_style(Style::new().blue())
                .direction(ListDirection::BottomToTop),
            results,
            buf,
            &mut ListState::default().with_selected(Some(0)),
        );
    }
}
