use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::StatefulWidget,
};

#[derive(Debug, Default)]
pub struct State {}

#[derive(Debug)]
pub struct MailBoxListWidget {}

impl MailBoxListWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for MailBoxListWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        todo!()
    }
}

pub fn render(state: &State, rect: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100)])
        .split(rect);
}
