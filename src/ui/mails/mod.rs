use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::ui::mails::mailbox_list::MailBoxListWidget;

mod mailbox_list;
mod mails;
mod preview;

#[derive(Debug, Default)]
pub struct State {
    mailbox_list: mailbox_list::State,
    mails: mails::State,
    preview: preview::State,
}

pub fn render(state: &mut State, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(frame.area());

    frame.render_stateful_widget(MailBoxListWidget::new(), layout[0], &mut state.mailbox_list);
    mails::render(&state.mails, layout[1]);
    preview::render(&state.preview, layout[2]);
}
