use crate::{
    datasource::types::QueryWindow,
    types::{CachedMail, CachedMailbox},
    ui::{
        Loadable,
        mailfs::types::{MailEntry, MailboxEntry, column::Column},
    },
};
use ratatui::widgets::TableState;
use std::ops::Range;

pub const DEFAULT_SECTION_SIZE: usize = 32;

#[derive(Debug, Default)]
pub struct MailboxColumn {
    pub entries: Vec<MailboxColumnEntry>,
    pub state: TableState,

    amount_child_mailboxes: usize,
    total_threads: usize,
}

impl MailboxColumn {
    pub fn new(
        child_mailboxes: Vec<MailboxEntry>,
        total_threads: usize,
        init_mails: Vec<CachedMail>,
    ) -> Self {
        let mut column = Self::default();
        column.set_entries(total_threads, child_mailboxes, init_mails);
        column
    }

    pub fn set_entries(
        &mut self,
        total_threads: usize,
        mailboxes: Vec<MailboxEntry>,
        first_mails: Vec<CachedMail>,
    ) {
        debug_assert!(first_mails.len() <= total_threads);

        let amount_child_mailboxes = mailboxes.len();

        let entries: Vec<MailboxColumnEntry> = {
            let mut entries = Vec::with_capacity(amount_child_mailboxes + total_threads);

            entries.extend(mailboxes.into_iter().map(MailboxColumnEntry::Mailbox));

            entries.extend(
                first_mails
                    .into_iter()
                    .map(MailEntry::from)
                    .map(Loadable::Loaded)
                    .map(MailboxColumnEntry::RootMail),
            );

            entries.resize_with(amount_child_mailboxes + total_threads, || {
                MailboxColumnEntry::RootMail(Loadable::NotLoaded)
            });

            entries
        };

        self.amount_child_mailboxes = amount_child_mailboxes;
        self.total_threads = total_threads;
        self.entries = entries;

        match self.state.selected() {
            Some(idx) => {
                if self.entries.is_empty() {
                    self.state.select(None);
                } else {
                    if idx >= self.entries.len() {
                        self.state.select(Some(self.entries.len() - 1));
                    }
                }
            }
            None => {
                if !self.entries.is_empty() {
                    self.state.select(Some(0));
                }
            }
        }
    }

    pub fn set_mailboxes(&mut self, mailboxes: Vec<CachedMailbox>) {
        let range = 0..self.amount_child_mailboxes;
        self.amount_child_mailboxes = mailboxes.len();

        let new_entries = mailboxes
            .into_iter()
            .map(MailboxEntry::from)
            .map(MailboxColumnEntry::Mailbox);

        self.entries.splice(range, new_entries);
    }

    pub fn set_mails(&mut self, window: QueryWindow, result: color_eyre::Result<Vec<CachedMail>>) {
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

    pub fn get_min_start_max_end(&self, section_size: Option<usize>) -> Range<usize> {
        let section_size = section_size.unwrap_or(DEFAULT_SECTION_SIZE);

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

        min_start..max_end
    }

    pub fn ensure_loaded_mails(&mut self, section_size: Option<usize>) -> Vec<QueryWindow> {
        let mut query_windows = Vec::new();

        let selected_idx = self.state.selected().unwrap_or(0);
        let max_range = self.get_min_start_max_end(section_size);

        // 1. check above
        if let Some(start_offset) = self.entries[max_range.start..selected_idx]
            .iter()
            .rev()
            .position(|entry| matches!(entry, MailboxColumnEntry::RootMail(root_mail) if root_mail.loaded().is_none()))
        {
            let range = max_range.start..selected_idx - start_offset;

            self.entries[range.clone()].fill(MailboxColumnEntry::RootMail(Loadable::Loading));

            query_windows.push(QueryWindow {
                start: range.start as u32,
                limit: range.len(),
            })
        }

        // 2. check below
        if let Some(start_offset) = self.entries[selected_idx..max_range.end]
            .iter()
            .position(|entry| matches!(entry, MailboxColumnEntry::RootMail(root_mail) if root_mail.loaded().is_none()))
        {
            let range = selected_idx + start_offset..max_range.end;

            self.entries[range.clone()].fill(MailboxColumnEntry::RootMail(Loadable::Loading));

            query_windows.push(QueryWindow {
                start: range.start as u32,
                limit: range.len(),
            });
        }

        tracing::debug!("Queries: {:#?}", query_windows);

        query_windows
    }

    pub fn get_amount_mailboxes(&self) -> usize {
        self.amount_child_mailboxes
    }
}

impl Column<MailboxColumnEntry> for MailboxColumn {
    type SelectedEntryResult<T> = Option<(usize, T)>;

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

    fn get_entry(&self, idx: usize) -> Option<&MailboxColumnEntry> {
        self.entries.get(idx)
    }

    fn get_selected_entry(&self) -> Option<(usize, &MailboxColumnEntry)> {
        self.state
            .selected()
            .and_then(|idx| self.entries.get(idx).map(|entry| (idx, entry)))
    }

    fn get_selected_entry_mut(&mut self) -> Option<(usize, &mut MailboxColumnEntry)> {
        self.state
            .selected()
            .and_then(|idx| self.entries.get_mut(idx).map(|entry| (idx, entry)))
    }
}

#[derive(Debug, Clone)]
pub enum MailboxColumnEntry {
    Mailbox(MailboxEntry),
    RootMail(Loadable<MailEntry>),
}
