use std::ops::Range;

use crate::{
    datasources::{
        hashmap::mail::QueryError,
        types::{QueryResponse, QueryState, QueryWindow},
    },
    types::MailId,
};

type StartPos = usize;

pub struct RootMails {
    sections: Vec<Section>,
    state: QueryState,
}

impl RootMails {
    pub fn new(state: QueryState) -> Self {
        Self {
            sections: Vec::new(),
            state,
        }
    }

    pub fn set(&mut self, start: StartPos, ids: &[MailId]) {
        if ids.is_empty() {
            return;
        }

        let end = start + ids.len();

        let section_a_idx = self
            .sections
            .partition_point(|section| section.pos_range().end < start);
        let section_b_idx = self
            .sections
            .partition_point(|section| section.start <= end);

        let not_in_any_section = section_a_idx == section_b_idx;
        if not_in_any_section {
            self.sections.insert(
                section_a_idx,
                Section {
                    start,
                    ids: ids.to_vec(),
                },
            );
            return;
        }

        let section_a_merged_start = self.sections[section_a_idx].start.min(start);
        let section_b_merged_end = self.sections[section_b_idx - 1].pos_range().end.max(end);

        let mut merged_section = Vec::with_capacity(section_b_merged_end - section_a_merged_start);

        // add prefix from `section_a`
        if section_a_merged_start < start {
            let section_a = &self.sections[section_a_idx];
            merged_section.extend_from_slice(&section_a.ids[..(start - section_a.start)]);
        }

        // add our new ones
        merged_section.extend_from_slice(ids);

        // add suffix from next `section_b`
        if end < section_b_merged_end {
            let section = &self.sections[section_b_idx - 1];
            merged_section.extend_from_slice(&section.ids[(end - section.start)..]);
        }

        self.sections.splice(
            section_a_idx..section_b_idx,
            [Section {
                start: section_a_merged_start,
                ids: merged_section,
            }],
        );
    }

    pub fn query(&self, window: QueryWindow) -> Result<QueryResponse<MailId>, QueryError> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Section {
    start: StartPos,
    ids: Vec<MailId>,
}

impl Section {
    pub fn pos_range(&self) -> Range<usize> {
        let start = self.start;
        let end = start + self.ids.len();

        start..end
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Section {
        fn test_new(range: Range<usize>) -> Self {
            let ids = generate_mail_ids(range.clone());

            Self {
                start: range.start,
                ids,
            }
        }
    }

    fn generate_mail_ids(range: Range<usize>) -> Vec<MailId> {
        range
            .clone()
            .into_iter()
            .map(|id| MailId(format!("{}", id)))
            .collect()
    }

    mod set {
        use super::*;

        #[test]
        fn new() {
            let mut mails = RootMails {
                sections: vec![Section::test_new(0..10), Section::test_new(30..40)],
                state: "1".into(),
            };

            let ids = generate_mail_ids(15..25);
            mails.set(15, &ids);

            assert_eq!(
                mails.sections,
                vec![
                    Section::test_new(0..10),
                    Section::test_new(15..25),
                    Section::test_new(30..40)
                ]
            )
        }

        #[test]
        fn merge_previous_section() {
            let mut mails = RootMails {
                sections: vec![Section::test_new(0..10), Section::test_new(30..40)],
                state: "1".into(),
            };

            let ids = generate_mail_ids(10..28);
            mails.set(10, &ids);

            assert_eq!(
                mails.sections,
                vec![Section::test_new(0..28), Section::test_new(30..40)]
            );
        }

        #[test]
        fn merge_next_section() {
            let mut mails = RootMails {
                sections: vec![Section::test_new(0..10), Section::test_new(30..40)],
                state: "1".into(),
            };

            let ids = generate_mail_ids(11..30);
            mails.set(11, &ids);

            assert_eq!(
                mails.sections,
                vec![Section::test_new(0..10), Section::test_new(11..40),]
            );
        }

        #[test]
        fn merge_other_two_sections() {
            let mut mails = RootMails {
                sections: vec![
                    Section::test_new(0..10),
                    Section::test_new(20..30),
                    Section::test_new(40..50),
                    Section::test_new(60..70),
                ],
                state: "1".into(),
            };

            let ids = generate_mail_ids(30..40);
            mails.set(30, &ids);

            assert_eq!(
                mails.sections,
                vec![
                    Section::test_new(0..10),
                    Section::test_new(20..50), // the inner should be merged
                    Section::test_new(60..70)
                ]
            );
        }
    }
}
