mod action;
mod list;
mod state;

use crate::backend;
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::HorizontalAlignment,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Mailboxes {
    state: state::State,
}

impl Mailboxes {
    pub async fn new(account: Arc<backend::Account>) -> Self {
        Self {
            state: state::State::new(account),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        let mut actions = Vec::new();

        match event.code {
            KeyCode::Char('q') => actions.push(super::Action::Quit),
            KeyCode::Char('j') => actions.push(Action::SelectNextMailbox.into()),
            KeyCode::Char('k') => actions.push(Action::SelectPreviousMailbox.into()),
            KeyCode::Enter => actions.push(Action::OpenSelectedMailbox.into()),
            _ => {}
        }

        actions
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => todo!(),
            Action::CloseCommandPalette => todo!(),

            Action::SelectNextMailbox => self.state.select_next_mailbox(),
            Action::SelectPreviousMailbox => self.state.select_previous_mailbox(),
            Action::OpenSelectedMailbox => {
                if let Some(selected_mailbox) = self.state.get_mut_selected_mailbox() {
                    return Some(super::Action::OpenMailList(
                        selected_mailbox.take_id().clone(),
                    ));
                }
            }
        }

        None
    }
}

impl Widget for &mut Mailboxes {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if let Some(mailboxes) = self.state.get_mailboxes() {
            StatefulWidget::render(
                list::List::new(&mailboxes).block(
                    Block::new()
                        .title("Mailboxes")
                        .title_alignment(HorizontalAlignment::Center),
                ),
                area,
                buf,
                self.state.get_list_state(),
            )
        } else {
            Widget::render(
                Paragraph::new("Loading mailboxes...")
                    .block(Block::bordered())
                    .centered(),
                area,
                buf,
            );
        }
    }
}
