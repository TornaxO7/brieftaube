use std::ops::Range;

use crate::{
    datasources::types::{QueryResponse, QueryState, QueryWindow},
    types::MailId,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {}

pub struct RootMails {
    sections: Vec<Section>,
    state: QueryState,
}

impl RootMails {
    pub fn new(start: usize, ids: Vec<MailId>, state: QueryState) -> Self {
        let section = Section { start, ids };

        Self {
            sections: vec![section],
            state,
        }
    }

    pub fn set(&mut self, start: usize, ids: Vec<MailId>, new_state: QueryState) {
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

        self.state = new_state;
    }

    pub fn query(&self, window: QueryWindow) -> QueryResponse<MailId> {
        todo!()
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
            root.set(0, ids, "1".into());

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
            root.set(20, ids, "1".into());

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
            root.set(20, ids, "1".into());

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
            root.set(10, ids, "1".into());

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
            root.set(30, ids, "1".into());

            assert_eq!(
                root.sections,
                vec![Section::new_test(0..10), Section::new_test(30..50)]
            );
        }
    }
}
