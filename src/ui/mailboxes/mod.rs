mod action;
mod list;
mod state;

use crate::{backend, ui::keybindmanager::KeybindManager};
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::HorizontalAlignment,
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct Mailboxes {
    state: state::State,

    keybindings: KeybindManager<super::Action>,
}

impl Mailboxes {
    pub async fn new(account: Arc<backend::Account>) -> Self {
        Self {
            state: state::State::new(account),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit.into()),
                ("j", Action::SelectNextMailbox.into()),
                ("k", Action::SelectPreviousMailbox.into()),
                ("n", super::Action::OpenComposer),
                ("<CR>", Action::OpenSelectedMailbox.into()),
                ("l", Action::OpenSelectedMailbox.into()),
            ])),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        match self.keybindings.handle_event(event) {
            Some(action) => vec![action],
            None => vec![],
        }
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
                    assert!(selected_mailbox.id().is_some());
                    return Some(super::Action::OpenMailList(Some(
                        selected_mailbox.id().unwrap().to_string(),
                    )));
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
        self.state.update();

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
