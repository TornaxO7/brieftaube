use crate::{
    datasource::{
        RootMailsCache,
        hashmap::HashMapDataSource,
        types::{QueryState, cache},
    },
    types::{MailId, MailboxId},
};
use async_trait::async_trait;
use color_eyre::Result;
use std::{collections::HashSet, ops::Range};

#[async_trait]
impl RootMailsCache for HashMapDataSource {
    async fn get_root_mails_state(&self, mailbox: &MailboxId) -> Option<&QueryState> {
        self.root_mails_state.get(mailbox)
    }

    async fn set_root_mails_state(
        &mut self,
        mailbox: &MailboxId,
        new_state: QueryState,
    ) -> Result<()> {
        self.root_mails_state
            .entry(mailbox.clone())
            .and_modify(|state| *state = new_state.clone())
            .or_insert(new_state);

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
    ) -> Result<Option<cache::QueryResponse<MailId>>> {
        let range = window.as_range();
        let Some(root_mails) = self.root_mails.get(mailbox) else {
            return Ok(None);
        };

        Ok(Some(root_mails.query(range)))
    }

    async fn insert_root_mails(
        &mut self,
        mailbox: &MailboxId,
        mails: Vec<(MailId, usize)>,
    ) -> Result<()> {
        match self.root_mails.entry(mailbox.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let root_mails = entry.get_mut();
                root_mails.add(mails);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut root_mails = RootMails::new();
                root_mails.add(mails);
                entry.insert(root_mails);
            }
        }

        Ok(())
    }

    async fn evict_root_mails(&mut self, mailbox: &MailboxId, ids: HashSet<MailId>) -> Result<()> {
        if let Some(root_mails) = self.root_mails.get_mut(mailbox) {
            root_mails.remove(ids);
        }

        Ok(())
    }
}

pub struct RootMails {
    ids: Vec<Option<MailId>>,
}

impl RootMails {
    pub fn new() -> Self {
        Self {
            ids: Vec::with_capacity(1024),
        }
    }

    pub fn add<MailIdsWithIndex>(&mut self, ids: MailIdsWithIndex)
    where
        MailIdsWithIndex: IntoIterator<Item = (MailId, usize)>,
    {
        for (id, index) in ids {
            if self.ids.len() < index {
                self.ids.resize(index, None);
            }

            self.ids.insert(index, Some(id));
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

    pub fn query(&self, range: Range<usize>) -> cache::QueryResponse<MailId> {
        debug_assert!(!range.is_empty());

        if self.ids.len() <= range.start {
            return cache::QueryResponse {
                values: vec![],
                missing: vec![range],
            };
        }

        let end = range.end.min(self.ids.len());

        let mut sections = Vec::new();
        let mut missing = Vec::new();

        let mut start = range.start;
        for chunk in self.ids[range.start..end].chunk_by(|a, b| a.is_some() == b.is_some()) {
            let is_section = chunk[0].is_some();
            if is_section {
                sections.push(cache::QueryResponseSection {
                    start,
                    values: chunk.iter().flatten().cloned().collect(),
                });
            } else {
                missing.push(start..start + chunk.len());
            }

            start += chunk.len();
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

    pub fn get_last_id(&self) -> Option<MailId> {
        self.ids
            .iter()
            .rev()
            .find(|id| id.is_some())
            .cloned()
            .flatten()
    }

    pub fn len(&self) -> usize {
        self.ids.len()
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
