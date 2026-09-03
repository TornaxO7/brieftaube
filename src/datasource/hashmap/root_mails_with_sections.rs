use crate::{
    datasource::{
        RootMailsCache,
        hashmap::HashMapDataSource,
        types::{QueryState, cache},
    },
    types::{MailData, MailId, MailboxId},
};
use std::ops::Range;

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
    sections: Vec<Section>,
    state: QueryState,
}

impl RootMails {
    pub fn new(start: usize, ids: Vec<MailId>, state: QueryState) -> Self {
        let section = Section { start, ids };

        Self {
            sections: vec![section],
            state: state,
        }
    }

    pub fn add(&mut self, ids: Vec<(MailId, usize)>) {
        for (id, index) in ids {
            self.set(index, vec![id]);
        }
    }

    pub fn set(&mut self, start: usize, ids: Vec<MailId>) {
        debug_assert!(!ids.is_empty());
        let end = start + ids.len();

        let section_a_idx = self
            .sections
            .partition_point(|section| section.end() < start);
        let section_b_idx = self
            .sections
            .partition_point(|section| section.start <= end);

        let no_overlapping_sections = section_a_idx == section_b_idx;
        if no_overlapping_sections {
            let new_section = Section { start, ids };
            self.sections.insert(section_a_idx, new_section);
            return;
        }

        let new_section_start = self.sections[section_a_idx].start.min(start);
        let new_section_end = self.sections[section_b_idx - 1].end().max(end);
        let mut merged_ids = Vec::with_capacity(new_section_end - new_section_start);

        if new_section_start < start {
            let section = &self.sections[section_a_idx];
            merged_ids.extend_from_slice(&section.ids[..start - section.start]);
        }

        merged_ids.extend_from_slice(&ids);

        if end < new_section_end {
            let section = &self.sections[section_b_idx - 1];
            merged_ids.extend_from_slice(&section.ids[end - section.start..]);
        }

        self.sections.splice(
            section_a_idx..section_b_idx,
            [Section {
                start: new_section_start,
                ids: merged_ids,
            }],
        );
    }

    pub fn query(&self, range: Range<usize>) -> cache::QueryResponse<MailId> {
        let mut sections = Vec::new();
        let mut missing = Vec::new();
        let mut cursor = range.start;
        let start_section = self
            .sections
            .partition_point(|section| section.end() <= cursor);

        for section in &self.sections[start_section..] {
            if section.start >= range.end {
                break;
            }

            if cursor < section.start {
                missing.push(cursor..section.start);
                cursor = section.start;
            }

            let section_range = {
                let start = cursor - section.start;
                let end = (range.end - section.start).min(section.ids.len());
                start..end
            };

            sections.push(cache::QueryResponseSection {
                start: cursor,
                values: section.ids[section_range.clone()].to_vec(),
            });

            cursor = section.start + section_range.end;
        }

        if cursor < range.end {
            missing.push(cursor..range.end);
        }

        cache::QueryResponse {
            values: sections,
            missing,
        }
    }

    pub fn flush(&mut self) {
        self.sections.clear();
    }

    pub fn state(&self) -> &QueryState {
        &self.state
    }

    pub fn set_state(&mut self, new_state: QueryState) {
        self.state = new_state;
    }

    pub fn get_last_id(&self) -> Option<MailId> {
        self.sections
            .last()
            .and_then(|last_section| last_section.ids.last().cloned())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Section {
    pub start: usize,
    pub ids: Vec<MailId>, // consecutive ids
}

impl Section {
    pub fn range(&self) -> Range<usize> {
        let start = self.start;
        let end = self.start + self.ids.len();

        start..end
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn end(&self) -> usize {
        self.range().end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_mail_ids(range: Range<usize>) -> Vec<MailId> {
        range
            .into_iter()
            .map(|id| MailId(format!("{}", id)))
            .collect()
    }

    impl Section {
        fn new_test(range: Range<usize>) -> Self {
            let start = range.start;
            let ids = new_mail_ids(range.clone());

            Self { start, ids }
        }
    }

    mod set {
        use super::*;

        #[test]
        fn new_section_in_beginning() {
            let mut root = RootMails {
                sections: vec![Section::new_test(10..20)],
                state: "1".into(),
            };

            let ids = new_mail_ids(0..5);
            root.set(0, ids);

            assert_eq!(
                root.sections,
                vec![Section::new_test(0..5), Section::new_test(10..20),]
            );
        }

        #[test]
        fn new_section_in_middle() {
            let mut root = RootMails {
                sections: vec![Section::new_test(0..10), Section::new_test(40..50)],
                state: "1".into(),
            };

            let ids = new_mail_ids(20..30);
            root.set(20, ids);

            assert_eq!(
                root.sections,
                vec![
                    Section::new_test(0..10),
                    Section::new_test(20..30),
                    Section::new_test(40..50),
                ]
            );
        }

        #[test]
        fn new_section_at_end() {
            let mut root = RootMails {
                sections: vec![Section::new_test(0..10)],
                state: "1".into(),
            };

            let ids = new_mail_ids(20..30);
            root.set(20, ids);

            assert_eq!(
                root.sections,
                vec![Section::new_test(0..10), Section::new_test(20..30)]
            );
        }

        #[test]
        fn merge_with_prev() {
            let mut root = RootMails {
                sections: vec![Section::new_test(0..10), Section::new_test(40..50)],
                state: "1".into(),
            };

            let ids = new_mail_ids(10..20);
            root.set(10, ids);

            assert_eq!(
                root.sections,
                vec![Section::new_test(0..20), Section::new_test(40..50)]
            );
        }

        #[test]
        fn merge_with_next() {
            let mut root = RootMails {
                sections: vec![Section::new_test(0..10), Section::new_test(40..50)],
                state: "1".into(),
            };

            let ids = new_mail_ids(30..40);
            root.set(30, ids);

            assert_eq!(
                root.sections,
                vec![Section::new_test(0..10), Section::new_test(30..50)]
            );
        }
    }

    mod query {
        use super::*;

        #[test]
        fn full_find() {
            let root = RootMails {
                sections: vec![Section::new_test(0..10)],
                state: "1".into(),
            };

            let query = root.query(2..5);

            assert_eq!(
                query.values,
                vec![cache::QueryResponseSection {
                    start: 2,
                    values: new_mail_ids(2..5),
                }]
            );

            assert!(query.missing.is_empty());
        }

        #[test]
        fn missing_first() {
            let root = RootMails {
                sections: vec![Section::new_test(10..20)],
                state: "1".into(),
            };

            let query = root.query(5..15);

            assert_eq!(
                query.values,
                vec![cache::QueryResponseSection {
                    start: 10,
                    values: new_mail_ids(10..15),
                }],
            );

            assert_eq!(query.missing, vec![5..10]);
        }

        #[test]
        fn missing_end() {
            let root = RootMails {
                sections: vec![Section::new_test(10..20)],
                state: "1".into(),
            };

            let query = root.query(15..25);

            assert_eq!(
                query.values,
                vec![cache::QueryResponseSection {
                    start: 15,
                    values: new_mail_ids(15..20),
                }],
            );

            assert_eq!(query.missing, vec![20..25]);
        }

        #[test]
        fn missing_between_sections() {
            let root = RootMails {
                sections: vec![Section::new_test(0..10), Section::new_test(40..50)],
                state: "1".into(),
            };

            let query = root.query(20..30);

            assert_eq!(query.values, vec![]);
            assert_eq!(query.missing, vec![20..30]);
        }

        #[test]
        fn with_empty_cache() {
            let root = RootMails {
                sections: vec![],
                state: "1".into(),
            };

            let query = root.query(42..69);

            assert_eq!(query.values, vec![]);
            assert_eq!(query.missing, vec![42..69]);
        }

        #[test]
        fn get_multiple() {
            let root = RootMails {
                sections: vec![Section::new_test(10..20), Section::new_test(30..40)],
                state: "1".into(),
            };

            let query = root.query(15..35);

            assert_eq!(
                query.values,
                vec![
                    cache::QueryResponseSection {
                        start: 15,
                        values: new_mail_ids(15..20)
                    },
                    cache::QueryResponseSection {
                        start: 30,
                        values: new_mail_ids(30..35)
                    }
                ],
            );

            assert_eq!(query.missing, vec![20..30]);
        }
    }
}
