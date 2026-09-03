use crate::{
    datasource::{
        RootMailsCache,
        hashmap::HashMapDataSource,
        types::{QueryState, cache},
    },
    types::{MailData, MailId, MailboxId},
};
use std::{collections::HashSet, ops::Range};

impl RootMailsCache for HashMapDataSource {
    async fn get_root_mails_state(&self, mailbox: &MailboxId) -> Option<&QueryState> {
        self.root_mails
            .get(mailbox)
            .map(|root_mails| root_mails.state())
    }

    async fn set_root_mails_state(
        &mut self,
        mailbox: &MailboxId,
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        if let Some(root_mails) = self.root_mails.get_mut(mailbox) {
            root_mails.set_state(new_state);
        }

        Ok(())
    }

    async fn get_root_mails_last_id(&self, mailbox: &MailboxId) -> Option<MailId> {
        self.root_mails
            .get(mailbox)
            .and_then(|root_mails| root_mails.get_last_id())
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: crate::datasource::types::QueryWindow,
    ) -> Result<Option<cache::QueryResponse<MailData>>, Self::Error> {
        let range = window.as_range();
        let Some(root_mails) = self.root_mails.get(mailbox) else {
            return Ok(None);
        };

        Ok(Some(root_mails.query(range).map(|id| {
            self.mails
                .get(&id)
                .cloned()
                .expect("MailData has been fetched as well")
        })))
    }

    async fn upsert_root_mails(
        &mut self,
        mailbox: &MailboxId,
        start: usize,
        root_mails: Vec<MailData>,
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        let root_mail_ids = root_mails.iter().map(|data| data.id.clone()).collect();

        match self.root_mails.entry(mailbox.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let root_mails = entry.get_mut();
                root_mails.set(start, root_mail_ids);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(RootMails::new(start, root_mail_ids, new_state));
            }
        }

        for root_mail in root_mails {
            let id = root_mail.id.clone();
            self.mails.insert(id, root_mail);
        }

        Ok(())
    }
}

pub struct RootMails {
    ids: Vec<Option<MailId>>,
    state: QueryState,
}

impl RootMails {
    pub fn new(start: usize, init_ids: Vec<MailId>, state: QueryState) -> Self {
        let end = start + init_ids.len();
        let mut ids = vec![None; end.next_power_of_two()];

        ids.splice(start..end, init_ids.into_iter().map(|id| Some(id)));

        Self { ids, state: state }
    }

    pub fn add(&mut self, ids: Vec<(MailId, usize)>) {
        for (id, index) in ids {
            self.set(index, vec![id]);
        }
    }

    pub fn remove(&mut self, mut ids: HashSet<MailId>) {
        for opt_id in self.ids.iter_mut() {
            if let Some(id) = opt_id {
                if ids.contains(id) {
                    ids.remove(&id);
                    *opt_id = None;
                }
            }
        }

        if !ids.is_empty() {
            if let Some(first_none) = self.ids.iter().position(|opt_id| opt_id.is_none()) {
                for opt_id in self.ids[first_none..].iter_mut() {
                    *opt_id = None;
                }
            }
        }
    }

    pub fn set(&mut self, start: usize, ids: Vec<MailId>) {
        debug_assert!(!ids.is_empty());
        let end = start + ids.len();
        if self.ids.len() < end {
            self.ids.resize(end.next_power_of_two(), None);
        }

        self.ids
            .splice(start..end, ids.into_iter().map(|id| Some(id)));
    }

    pub fn query(&self, range: Range<usize>) -> cache::QueryResponse<MailId> {
        if self.ids.len() <= range.start {
            return cache::QueryResponse {
                values: vec![],
                missing: vec![range],
            };
        }

        let end = range.end.min(self.ids.len());

        let mut sections = Vec::new();
        let mut missing = Vec::new();

        let mut prev_start = range.start;
        for (idx, value) in self.ids[range.start..end].iter().enumerate().skip(1) {
            match (&self.ids[idx - 1], value) {
                (None, None) | (Some(_), Some(_)) => {}
                (None, Some(_)) => {
                    missing.push(prev_start..idx);
                    prev_start = idx;
                }
                (Some(_), None) => {
                    sections.push(cache::QueryResponseSection {
                        start: prev_start,
                        values: self.ids[prev_start..idx]
                            .iter()
                            .cloned()
                            .map(|id| id.unwrap())
                            .collect(),
                    });
                    prev_start = idx;
                }
            }
        }

        if prev_start < end {
            if self.ids[prev_start].is_some() {
                sections.push(cache::QueryResponseSection {
                    start: prev_start,
                    values: self.ids[prev_start..end]
                        .iter()
                        .cloned()
                        .map(|id| id.unwrap())
                        .collect(),
                });
            } else {
                missing.push(prev_start..end);
            }
        }

        if end < range.end {
            missing.push(end..range.end);
        }

        cache::QueryResponse {
            values: sections,
            missing,
        }
    }

    pub fn flush(&mut self) {
        self.ids.clear();
    }

    pub fn state(&self) -> &QueryState {
        &self.state
    }

    pub fn set_state(&mut self, new_state: QueryState) {
        self.state = new_state;
    }

    pub fn get_last_id(&self) -> Option<MailId> {
        self.ids
            .iter()
            .rev()
            .find(|id| id.is_some())
            .cloned()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod query {
        use super::*;

        #[test]
        fn all_some() {
            let root = RootMails {
                ids: vec![Some("1".into()), Some("2".into())],
                state: "1".into(),
            };

            assert_eq!(
                root.query(0..2),
                cache::QueryResponse {
                    values: vec![cache::QueryResponseSection {
                        start: 0,
                        values: vec!["1".into(), "2".into()]
                    }],
                    missing: vec![]
                }
            )
        }

        #[test]
        fn all_none() {
            let root = RootMails {
                ids: vec![None, None],
                state: "1".into(),
            };

            assert_eq!(
                root.query(0..2),
                cache::QueryResponse {
                    values: vec![],
                    missing: vec![0..2]
                }
            );
        }

        #[test]
        fn empty_prefix_and_suffix() {
            let root = RootMails {
                ids: vec![None, Some("1".into()), Some("2".into()), None],
                state: "1".into(),
            };

            assert_eq!(
                root.query(0..4),
                cache::QueryResponse {
                    values: vec![cache::QueryResponseSection {
                        start: 1,
                        values: vec!["1".into(), "2".into()]
                    }],
                    missing: vec![0..1, 3..4]
                }
            );
        }

        #[test]
        fn empty_infix() {
            let root = RootMails {
                ids: vec![Some("1".into()), None, Some("2".into())],
                state: "1".into(),
            };

            assert_eq!(
                root.query(0..3),
                cache::QueryResponse {
                    values: vec![
                        cache::QueryResponseSection {
                            start: 0,
                            values: vec!["1".into()]
                        },
                        cache::QueryResponseSection {
                            start: 2,
                            values: vec!["2".into()]
                        }
                    ],
                    missing: vec![1..2]
                }
            );
        }

        #[test]
        fn range_bigger_than_values() {
            let root = RootMails {
                ids: vec![Some("1".into())],
                state: "1".into(),
            };

            assert_eq!(
                root.query(0..5),
                cache::QueryResponse {
                    values: vec![cache::QueryResponseSection {
                        start: 0,
                        values: vec!["1".into()]
                    }],
                    missing: vec![1..5]
                }
            );
        }
    }
}
