use crate::{
    datasource::types::QueryWindow,
    types::{CachedMail, CachedMailbox},
    ui::{
        Loadable,
        mailfs::types::{MailEntry, MailboxEntry, column::Column},
    },
};
use ratatui::widgets::TableState;

pub const DEFAULT_SECTION_SIZE: usize = 32;

#[derive(Debug)]
pub struct MailboxColumn {
    pub entries: Vec<MailboxColumnEntry>,
    pub state: TableState,

    amount_child_mailboxes: usize,
    total_threads: usize,
}

impl MailboxColumn {
    pub fn new(
        mut child_mailboxes: Vec<CachedMailbox>,
        total_threads: usize,
        init_mails: Vec<MailEntry>,
    ) -> Self {
        debug_assert!(init_mails.len() <= total_threads);

        child_mailboxes.sort_by(|a, b| {
            if a.sort_order != b.sort_order {
                a.sort_order.cmp(&b.sort_order)
            } else {
                a.name.cmp(&b.name)
            }
        });

        let amount_child_mailboxes = child_mailboxes.len();

        let state = {
            let state = TableState::new();

            if !child_mailboxes.is_empty() || init_mails.len() > 0 {
                state.with_selected(Some(0))
            } else {
                state
            }
        };

        let entries: Vec<MailboxColumnEntry> = {
            let mut entries = Vec::with_capacity(amount_child_mailboxes + total_threads);

            entries.extend(
                child_mailboxes
                    .into_iter()
                    .map(MailboxEntry::from)
                    .map(MailboxColumnEntry::Mailbox),
            );

            entries.extend(
                init_mails
                    .into_iter()
                    .map(Loadable::Loaded)
                    .map(MailboxColumnEntry::RootMail),
            );

            entries.resize_with(amount_child_mailboxes + total_threads, || {
                MailboxColumnEntry::RootMail(Loadable::NotLoaded)
            });

            entries
        };

        Self {
            entries,
            state,
            amount_child_mailboxes,
            total_threads,
        }
    }

    pub fn set_mailbox_mails(
        &mut self,
        window: QueryWindow,
        result: color_eyre::Result<Vec<CachedMail>>,
    ) {
        let window_range = {
            let range = window.as_range();
            self.amount_child_mailboxes + range.start..self.amount_child_mailboxes + range.end
        };

        match result {
            Ok(mails) => {
                let mail_entries: Vec<MailboxColumnEntry> = mails
                    .into_iter()
                    .map(|cached_mail| {
                        MailboxColumnEntry::RootMail(Loadable::Loaded(MailEntry::from(cached_mail)))
                    })
                    .collect();

                self.entries.splice(window_range, mail_entries);
            }
            Err(err) => {
                self.entries[window_range].fill(MailboxColumnEntry::RootMail(Loadable::Error(
                    err.to_string(),
                )));
            }
        };
    }

    pub fn ensure_loaded_mails(&mut self, section_size: Option<usize>) -> Vec<QueryWindow> {
        let section_size = section_size.unwrap_or(DEFAULT_SECTION_SIZE);
        let mut query_windows = Vec::new();

        // min_start                     max_end
        // |                             |
        // |---------|---------|---------|
        //           |         |
        //           `selected_idx` somewhere here
        let selected_idx = self.state.selected().unwrap_or(0);
        let max_end = (selected_idx.next_multiple_of(section_size) + section_size).min(self.len());
        let min_start = max_end
            .saturating_sub(section_size * 3)
            .max(self.amount_child_mailboxes);

        // 1. check above
        if let Some(start_offset) = self.entries[min_start..selected_idx]
            .iter()
            .rev()
            .position(|entry| matches!(entry, MailboxColumnEntry::RootMail(root_mail) if root_mail.loaded().is_none()))
        {
            let range = min_start..selected_idx - start_offset;

            self.entries[range.clone()].fill(MailboxColumnEntry::RootMail(Loadable::Loading));

            query_windows.push(QueryWindow {
                start: range.start as u32,
                limit: range.len(),
            })
        }

        // 2. check below
        if let Some(start_offset) = self.entries[selected_idx..max_end]
            .iter()
            .position(|entry| matches!(entry, MailboxColumnEntry::RootMail(root_mail) if root_mail.loaded().is_none()))
        {
            let range = selected_idx + start_offset..max_end;

            self.entries[range.clone()].fill(MailboxColumnEntry::RootMail(Loadable::Loading));

            query_windows.push(QueryWindow {
                start: range.start as u32,
                limit: range.len(),
            });
        }

        tracing::debug!("Queries: {:#?}", query_windows);

        query_windows
    }
}

impl Column<MailboxColumnEntry> for MailboxColumn {
    type ColumnEntryResult<T> = Option<T>;

    fn navigate_up(&mut self, offset: u16) {
        if let Some(idx) = self.state.selected() {
            let new_idx = idx.saturating_sub(offset as usize);
            self.state.select(Some(new_idx));
        }
    }

    fn navigate_down(&mut self, offset: u16) {
        if let Some(idx) = self.state.selected() {
            let new_idx = (idx + offset as usize).min(self.len() - 1);
            self.state.select(Some(new_idx));
        }
    }

    fn navigate_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.state.select(Some(self.len() - 1));
        }
    }

    fn navigate_to_top(&mut self) {
        if !self.entries.is_empty() {
            self.state.select(Some(0));
        }
    }

    fn len(&self) -> usize {
        self.amount_child_mailboxes + self.total_threads
    }

    fn get_selected_entry(&self) -> Option<&MailboxColumnEntry> {
        self.state.selected().and_then(|idx| self.entries.get(idx))
    }

    fn get_selected_entry_mut(&mut self) -> Option<&mut MailboxColumnEntry> {
        self.state
            .selected()
            .and_then(|idx| self.entries.get_mut(idx))
    }
}

#[derive(Debug, Clone)]
pub enum MailboxColumnEntry {
    Mailbox(MailboxEntry),
    RootMail(Loadable<MailEntry>),
}

// #[derive(Debug)]
// pub enum MailboxColumnEntry<'a> {
//     Mailbox(&'a MailboxEntry),
//     RootMail(&'a Loadable<MailEntry>),
// }

// #[derive(Debug)]
// pub enum MailboxColumnEntryMut<'a> {
//     Mailbox(&'a mut MailboxEntry),
//     RootMail(&'a mut Loadable<MailEntry>),
// }
