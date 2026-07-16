use crate::{mailboxes::backend::MailboxData, utils::MailboxId};
use ratatui::widgets::TableState;
use std::cmp::Ordering;

#[derive(Clone, Default)]
pub struct Layer {
    pub mailbox_owner: Option<MailboxId>,
    pub mailboxes: Vec<MailboxData>,
    pub state: TableState,
}

impl Layer {
    pub fn new(mailbox_owner: Option<MailboxId>) -> Self {
        Self {
            mailbox_owner,
            mailboxes: vec![],
            state: TableState::new().with_selected(Some(0)),
        }
    }

    pub fn is_root_layer(&self) -> bool {
        self.mailbox_owner.is_none()
    }

    pub fn selected_parent(&self) -> bool {
        !self.is_root_layer() && self.state.selected().map(|idx| idx == 0).unwrap()
    }

    pub fn contains_mailbox_name(&self, name: &str) -> bool {
        self.mailboxes.iter().any(|mailbox| mailbox.name == name)
    }

    pub fn get_mailbox(&self, id: &MailboxId) -> Option<&MailboxData> {
        self.mailboxes
            .iter()
            .find(|mailbox| mailbox.id == id.as_str())
    }

    pub fn get_mailbox_mut(&mut self, id: &MailboxId) -> Option<&mut MailboxData> {
        self.mailboxes
            .iter_mut()
            .find(|mailbox| mailbox.id == id.as_str())
    }

    pub fn get_selected_mailbox(&self) -> Option<&MailboxData> {
        if self.is_root_layer() {
            self.mailboxes.get(self.state.selected().unwrap())
        } else if self.selected_parent() {
            None
        } else {
            self.mailboxes.get(self.state.selected().unwrap() - 1)
        }
    }

    pub fn sort_mailboxes(&mut self) {
        self.mailboxes.sort_by(|a, b| {
            let ordering = a.sort_order.cmp(&b.sort_order);

            if ordering == Ordering::Equal {
                a.name.cmp(&b.name)
            } else {
                ordering
            }
        })
    }
}
