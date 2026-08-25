mod state;

use crate::backend::{
    Backend, ParentMailboxId, mailbox::types::MailboxId, mails::types::MailId,
    threads::types::ThreadId,
};
use ratatui::widgets::TableState;
use std::sync::Arc;

pub struct ColumnRemoveMissingEntry;
pub use state::*;

/// Internal representation of a column
#[derive(Clone, Debug)]
pub struct Column {
    mailbox: ParentMailboxId,
    /// The entries within the column
    entries: Vec<ColumnEntry>,
    /// The table state
    pub state: TableState,
}

impl Column {
    pub fn new(mailbox: ParentMailboxId, entries: Vec<ColumnEntry>) -> Self {
        let state = if entries.is_empty() {
            TableState::new()
        } else {
            TableState::new().with_selected(0)
        };

        Self {
            mailbox,
            entries,
            state,
        }
    }

    pub fn selected_idx(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_entry(&self) -> Option<&ColumnEntry> {
        self.state.selected().and_then(|idx| self.entries.get(idx))
    }

    pub fn entries(&self) -> &[ColumnEntry] {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<ColumnEntry> {
        &mut self.entries
    }

    // pub fn add_entry(&mut self, entry: ColumnEntry, backend: Arc<Backend>) {
    //     match entry {
    //         ColumnEntryDiff::Mailbox(mailbox_id) => self.add_mailbox(mailbox_id, backend),
    //         ColumnEntryDiff::SingleMail(mail_id) => self.add_single_mail(mail_id, backend),
    //         ColumnEntryDiff::ThreadMail { mail, thread } => {
    //             self.add_thread_mail(mail, thread, backend)
    //         }
    //     }
    // }

    // pub fn remove_entry(&mut self, backend: Arc<Backend>) -> Result<(), ColumnRemoveMissingEntry> {
    //     match entry {
    //         ColumnEntryDiff::Mailbox(mailbox_id) => self.remove_mailbox(mailbox_id),
    //         ColumnEntryDiff::SingleMail(mail_id) => self.remove_single_mail(mail_id),
    //         ColumnEntryDiff::ThreadMail { mail, thread } => {
    //             self.remove_thread_mail(mail, thread, backend)
    //         }
    //     }
    // }
}

// Adding entries
// impl Column {
//     fn add_mailbox(&mut self, id: MailboxId, backend: Arc<Backend>) {
//         let mailbox_to_add = backend.get_mailbox(&id).unwrap();

//         let add_idx = self
//             .entries
//             .iter()
//             .map_while(|entry| match entry {
//                 ColumnEntry::Mailbox(id) => Some(id),
//                 _ => None,
//             })
//             .position(|id| {
//                 let other = backend.get_mailbox(id).unwrap();
//                 other.sort_order > mailbox_to_add.sort_order
//             })
//             .unwrap_or_else(|| {
//                 self.entries
//                     .iter()
//                     .position(|entry| !matches!(entry, ColumnEntry::Mailbox(_)))
//                     .unwrap_or(0)
//             });

//         self.entries
//             .insert(add_idx, ColumnEntry::Mailbox(id.clone()));
//     }

//     fn add_single_mail(&mut self, id: MailId, backend: Arc<Backend>) {
//         let mail_to_add = backend.get_mail(&id).unwrap();
//         let add_idx = self
//             .entries
//             .iter()
//             .position(|entry| match entry {
//                 ColumnEntry::Mailbox(_)
//                 | ColumnEntry::ThreadStart { .. }
//                 | ColumnEntry::ThreadChild(_, _)
//                 | ColumnEntry::ThreadEnd(_, _) => false,

//                 ColumnEntry::SingleMail(mail_id) | ColumnEntry::CollapsedThread(mail_id, _) => {
//                     let other = backend.get_mail(mail_id).unwrap();
//                     other.received_at > mail_to_add.received_at
//                 }
//             })
//             .unwrap_or_else(|| {
//                 self.entries
//                     .iter()
//                     .position(|entry| !matches!(entry, ColumnEntry::Mailbox(_)))
//                     .unwrap_or(0)
//             });

//         self.entries.insert(add_idx, ColumnEntry::SingleMail(id));
//     }

//     fn add_thread_mail(&mut self, mail_id: MailId, thread_id: ThreadId, backend: Arc<Backend>) {
//         let mail_to_add = backend.get_mail(&mail_id).unwrap();
//         let thread_entry = self.entries.iter_mut().find_map(|entry| match entry {
//             ColumnEntry::Mailbox(_)
//             | ColumnEntry::SingleMail(_)
//             | ColumnEntry::ThreadChild(_, _)
//             | ColumnEntry::ThreadEnd(_, _) => None,

//             ColumnEntry::CollapsedThread(mail_id, _)
//             | ColumnEntry::ThreadStart {
//                 collapsed_mail_id: mail_id,
//                 ..
//             } => Some(mail_id),
//         });

//         match thread_entry {
//             Some(thread_mail_id) => {
//                 let thread_mail = backend.get_mail(thread_mail_id).unwrap();

//                 if mail_to_add.received_at > thread_mail.received_at {
//                     *thread_mail_id = mail_id;
//                 }
//             }
//             None => {
//                 let add_idx = self
//                     .entries
//                     .iter()
//                     .position(|entry| match entry {
//                         ColumnEntry::Mailbox(_)
//                         | ColumnEntry::ThreadStart { .. }
//                         | ColumnEntry::ThreadChild(_, _)
//                         | ColumnEntry::ThreadEnd(_, _) => false,

//                         ColumnEntry::SingleMail(mail_id)
//                         | ColumnEntry::CollapsedThread(mail_id, _) => {
//                             let other = backend.get_mail(mail_id).unwrap();
//                             mail_to_add.received_at > other.received_at
//                         }
//                     })
//                     .unwrap_or_else(|| {
//                         self.entries
//                             .iter()
//                             .position(|entry| !matches!(entry, ColumnEntry::Mailbox(_)))
//                             .unwrap_or(0)
//                     });

//                 self.entries
//                     .insert(add_idx, ColumnEntry::CollapsedThread(mail_id, thread_id));
//             }
//         }
//     }
// }

// removing entries
// impl Column {
//     fn remove_mailbox(&mut self, id: MailboxId) -> Result<(), ColumnRemoveMissingEntry> {
//         let idx = self
//             .entries
//             .iter()
//             .position(|entry| matches!(entry, ColumnEntry::Mailbox(other) if other == &id))
//             .ok_or(ColumnRemoveMissingEntry)?;

//         self.entries.remove(idx);
//         Ok(())
//     }

//     fn remove_single_mail(&mut self, id: MailId) -> Result<(), ColumnRemoveMissingEntry> {
//         let idx = self
//             .entries
//             .iter()
//             .position(|entry| matches!(entry, ColumnEntry::SingleMail(other) if other == &id))
//             .ok_or(ColumnRemoveMissingEntry)?;

//         self.entries.remove(idx);
//         Ok(())
//     }

//     fn remove_thread_mail(
//         &mut self,
//         mail_id: MailId,
//         thread_id: ThreadId,
//         backend: Arc<Backend>,
//     ) -> Result<(), ColumnRemoveMissingEntry> {
//         let column_mailbox = self.mailbox.as_ref().unwrap();

//         let (idx, old_starting_mail_id_of_thread) = self
//             .entries
//             .iter_mut()
//             .enumerate()
//             .find_map(|(idx, entry)| match entry {
//                 ColumnEntry::Mailbox(_)
//                 | ColumnEntry::SingleMail(_)
//                 | ColumnEntry::ThreadChild(_, _)
//                 | ColumnEntry::ThreadEnd(_, _) => None,
//                 ColumnEntry::CollapsedThread(other_mail_id, _)
//                 | ColumnEntry::ThreadStart {
//                     collapsed_mail_id: other_mail_id,
//                     ..
//                 } => (other_mail_id == &mail_id).then_some((idx, other_mail_id)),
//             })
//             .ok_or(ColumnRemoveMissingEntry)?;

//         let thread_mails = backend.get_thread_mail_ids(&thread_id).unwrap();

//         let next_thread_mail_in_mailbox = thread_mails.iter().rev().find(|thread_mail| {
//             let is_different_mail = thread_mail.id != mail_id;
//             let is_also_in_this_mailbox = thread_mail.mailbox_ids.contains(column_mailbox);

//             is_different_mail && is_also_in_this_mailbox
//         });

//         match next_thread_mail_in_mailbox {
//             Some(next) => {
//                 *old_starting_mail_id_of_thread = next.id.clone();
//             }
//             None => {
//                 self.entries.remove(idx);
//             }
//         };

//         Ok(())
//     }
// }

#[derive(Debug, Clone)]
pub enum ColumnEntryDiff {
    Mailbox(MailboxId),
    SingleMail(MailId),
    ThreadMail { mail: MailId, thread: ThreadId },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ColumnEntry {
    Mailbox(MailboxId),
    Mail(MailEntry),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MailEntry {
    pub mail_id: MailId,
    pub thread_id: ThreadId,
    pub thread_role: ThreadRole,
}

impl MailEntry {
    pub async fn new(id: MailId, backend: &Backend) -> Self {
        let mail = backend.get_mail(&id).await.unwrap();
        let thread = backend.get_thread(&mail.thread_id);

        let is_single_mail = thread.len() == 1;
        let thread_role = if is_single_mail {
            ThreadRole::Single
        } else {
            ThreadRole::Collapsed
        };

        Self {
            mail_id: mail.id,
            thread_id: mail.thread_id,
            thread_role,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreadRole {
    Single,
    Collapsed,
    ThreadStart,
    ThreadChild,
    ThreadEnd,
}
