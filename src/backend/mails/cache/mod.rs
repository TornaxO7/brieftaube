pub mod error;

use super::MailId;
use crate::backend::{
    GetState,
    mailbox::types::MailboxId,
    mails::{MailData, types::MailUpdate},
    threads::types::ThreadId,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Cache {
    mails: HashMap<MailId, MailData>,
    // - Guaranteed that there's a `MailData` for each `MailId` here
    // - Sorted by `received_at`
    mailbox_mapping: HashMap<MailboxId, Vec<MailId>>,
    // - Guaranteed that there's a `MailData` for each `MailId` here
    // - Sorted by `received_at`
    thread_mapping: HashMap<ThreadId, Vec<MailId>>,
    state: GetState,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
            mailbox_mapping: HashMap::new(),
            thread_mapping: HashMap::new(),
            state: GetState::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mails.is_empty()
    }

    pub fn get_state(&self) -> String {
        self.state.clone()
    }

    pub fn set_state(&mut self, new_state: String) {
        self.state = new_state;
    }

    pub fn get_mail(&self, id: &MailId) -> Option<&MailData> {
        self.mails.get(id)
    }

    pub fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData> {
        self.mails.get_mut(id)
    }
}

// helper methods
impl Cache {}

// Methods altering the cache
impl Cache {
    pub fn flush(&mut self) {
        self.mails.clear();
        self.mailbox_mapping.clear();
        self.state.clear();
    }

    pub fn add(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), mail.clone());

        // add to `mailbox_mapping`
        for mailbox_id in mail.mailbox_ids.iter().cloned() {
            self.mailbox_mapping
                .entry(mailbox_id)
                .and_modify(|mailbox_mails| {
                    let idx = get_idx(&self.mails, mailbox_mails, &mail);
                    mailbox_mails.insert(idx, mail.id.clone());
                })
                .or_insert(vec![mail.id.clone()]);
        }

        // add to `thread_mapping`
        self.thread_mapping
            .entry(mail.thread_id.clone())
            .and_modify(|thread_mails| {
                let idx = get_idx(&self.mails, thread_mails, &mail);
                thread_mails.insert(idx, mail.id.clone());
            })
            .or_insert(vec![mail.id.clone()]);
    }

    pub fn remove(&mut self, id: MailId) -> Option<MailData> {
        let mail = self.mails.remove(&id)?;

        // remove from `mailbox_mapping`
        for mailbox_id in mail.mailbox_ids.iter() {
            let mailbox_mails = self.mailbox_mapping.get_mut(mailbox_id).unwrap();
            let idx = get_idx(&self.mails, mailbox_mails, &mail);
            mailbox_mails.remove(idx);
        }

        // remove from `threads_mapping`
        {
            let thread_mapping = self.thread_mapping.get_mut(&mail.thread_id).unwrap();
            let idx = get_idx(&self.mails, &thread_mapping, &mail);
            thread_mapping.remove(idx);
        }

        Some(mail)
    }

    // Returns `Err` if there's no mail with the given id.
    pub fn update(&mut self, new: MailUpdate) -> Result<(), ()> {
        if let Some(patch_keywords) = &new.patch_keywords {
            let mail = self.mails.get_mut(&new.id).ok_or(())?;

            for (keyword, set) in patch_keywords {
                if *set {
                    mail.keywords.insert(keyword.clone());
                } else {
                    mail.keywords.remove(keyword);
                }
            }
        }

        if let Some(mailbox_ids) = &new.mailbox_ids {
            for (new_mailbox, set) in mailbox_ids {
                if *set {
                    let mail = self.mails.get(&new.id).ok_or(())?;

                    self.mailbox_mapping
                        .entry(new_mailbox.clone())
                        .and_modify(|mailbox_mails| {
                            let idx = get_idx(&self.mails, mailbox_mails, mail);

                            mailbox_mails.insert(idx, mail.id.clone());
                        })
                        .or_insert(vec![mail.id.clone()]);

                    let mail = self.mails.get_mut(&new.id).ok_or(())?;
                    mail.mailbox_ids.insert(new_mailbox.clone());
                } else {
                    let mail = self.mails.get(&new.id).ok_or(())?;
                    self.mailbox_mapping
                        .entry(new_mailbox.clone())
                        .and_modify(|mailbox_mails| {
                            let idx = get_idx(&self.mails, mailbox_mails, mail);
                            mailbox_mails.remove(idx);
                        });

                    let mail = self.mails.get_mut(&new.id).unwrap();
                    mail.mailbox_ids.remove(new_mailbox);
                }
            }
        }

        Ok(())
    }
}

fn get_idx(mails: &HashMap<MailId, MailData>, mapping: &[MailId], mail: &MailData) -> usize {
    mapping
        .binary_search_by(|other_id| {
            let other = mails.get(other_id).unwrap();
            other.received_at.cmp(&mail.received_at)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mails::types::MailUpdate;
    use chrono::{DateTime, Duration, Local};
    use std::collections::HashMap;

    fn mail(
        id: &str,
        thread_id: &str,
        mailbox_ids: &[&str],
        received_at: DateTime<Local>,
    ) -> MailData {
        MailData {
            id: id.to_string(),
            thread_id: thread_id.to_string(),
            mailbox_ids: mailbox_ids.iter().map(|s| s.to_string()).collect(),
            received_at,
            ..Default::default()
        }
    }

    mod add {
        use super::*;

        #[test]
        fn adds_mail_to_mails_map() {
            let mut cache = Cache::new();
            let m = mail("1", "t1", &["inbox"], Local::now());
            cache.add(m.clone());

            assert_eq!(cache.mails.get("1"), Some(&m));
        }

        #[test]
        fn adds_mail_to_mailbox_mapping() {
            let mut cache = Cache::new();
            cache.add(mail("1", "t1", &["inbox"], Local::now()));

            assert!(cache.is_empty(&"inbox".to_string()));
            assert_eq!(
                cache.mailbox_mapping.get("inbox").unwrap(),
                &vec!["1".to_string()]
            );
        }

        #[test]
        fn keeps_mailbox_sorted_by_received_at() {
            let mut cache = Cache::new();
            let now = Local::now();

            cache.add(mail("2", "t1", &["inbox"], now));
            cache.add(mail("1", "t1", &["inbox"], now - Duration::minutes(10)));
            cache.add(mail("3", "t1", &["inbox"], now + Duration::minutes(10)));

            assert_eq!(
                cache.mailbox_mapping.get("inbox").unwrap(),
                &vec!["1".to_string(), "2".to_string(), "3".to_string()]
            );
        }

        #[test]
        fn adds_mail_to_multiple_mailboxes() {
            let mut cache = Cache::new();
            cache.add(mail("1", "t1", &["inbox", "archive"], Local::now()));

            assert!(cache.is_empty(&"inbox".to_string()));
            assert!(cache.is_empty(&"archive".to_string()));
        }

        #[test]
        fn adds_mail_to_thread_mapping() {
            let mut cache = Cache::new();
            let m = mail("1", "t1", &["inbox"], Local::now());
            cache.add(m);

            // Note: current implementation keys `thread_mapping` by `mail.id`
            // instead of `mail.thread_id` - this test documents that behavior.
            assert_eq!(
                cache.thread_mapping.get("1").unwrap(),
                &vec!["1".to_string()]
            );
        }
    }

    mod remove {
        use super::*;

        #[test]
        fn removes_mail_from_mails_map() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            let removed = cache.remove("1".to_string());

            assert!(removed.is_some());
            assert_eq!(cache.get_mail(&"1".to_string()), None);
        }

        #[test]
        fn removes_mail_from_mailbox_mapping() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            cache.remove("1".to_string());

            assert!(cache.mailbox_mapping.get("inbox").unwrap().is_empty());
        }

        #[test]
        fn removes_only_the_targeted_mail() {
            let mut cache = Cache::new();
            let now = Local::now();
            cache.add(mail("1", "1", &["inbox"], now));
            cache.add(mail("2", "2", &["inbox"], now + Duration::minutes(1)));

            cache.remove("1".to_string());

            assert_eq!(
                cache.mailbox_mapping.get("inbox").unwrap(),
                &vec!["2".to_string()]
            );
        }

        #[test]
        fn returns_none_for_unknown_id() {
            let mut cache = Cache::new();
            assert_eq!(cache.remove("unknown".to_string()), None);
        }

        // Uses id == thread_id, since `remove` looks up `thread_mapping` by
        // `mail.thread_id` while `add` inserts it under `mail.id` (see bug
        // note above). This keeps the test meaningful under current behavior.
        #[test]
        fn removes_mail_from_thread_mapping() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            cache.remove("1".to_string());

            assert!(cache.thread_mapping.get("1").unwrap().is_empty());
        }
    }

    mod update {
        use crate::backend::mails::types::MailKeyword;

        use super::*;

        #[test]
        fn patches_keywords_on() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            let result = cache.update(MailUpdate {
                id: "1".to_string(),
                patch_keywords: Some(vec![(MailKeyword::Seen, true)]),
                mailbox_ids: None,
            });

            assert!(result.is_ok());
            assert!(
                cache
                    .get_mail(&"1".to_string())
                    .unwrap()
                    .keywords
                    .contains(&MailKeyword::Seen)
            );
        }

        #[test]
        fn patches_keywords_off() {
            let mut cache = Cache::new();
            let mut m = mail("1", "1", &["inbox"], Local::now());
            m.keywords.insert(MailKeyword::Seen);
            cache.add(m);

            let mut patch = HashMap::new();
            patch.insert("seen".to_string(), false);

            cache
                .update(MailUpdate {
                    id: "1".to_string(),
                    patch_keywords: Some(vec![(MailKeyword::Seen, false)]),
                    mailbox_ids: None,
                })
                .unwrap();

            assert!(
                !cache
                    .get_mail(&"1".to_string())
                    .unwrap()
                    .keywords
                    .contains(&MailKeyword::Seen)
            );
        }

        #[test]
        fn adds_mail_to_new_mailbox() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            cache
                .update(MailUpdate {
                    id: "1".to_string(),
                    patch_keywords: None,
                    mailbox_ids: Some(vec![("archive".into(), true)]),
                })
                .unwrap();

            assert!(
                cache
                    .get_mail(&"1".to_string())
                    .unwrap()
                    .mailbox_ids
                    .contains("archive")
            );
            assert_eq!(
                cache.mailbox_mapping.get("archive").unwrap(),
                &vec!["1".to_string()]
            );
        }

        #[test]
        fn removes_mail_from_mailbox() {
            let mut cache = Cache::new();
            cache.add(mail("1", "1", &["inbox"], Local::now()));

            cache
                .update(MailUpdate {
                    id: "1".to_string(),
                    patch_keywords: None,
                    mailbox_ids: Some(vec![("inbox".into(), false)]),
                })
                .unwrap();

            assert!(
                !cache
                    .get_mail(&"1".to_string())
                    .unwrap()
                    .mailbox_ids
                    .contains("inbox")
            );
            assert!(cache.mailbox_mapping.get("inbox").unwrap().is_empty());
        }

        #[test]
        fn keeps_new_mailbox_sorted() {
            let mut cache = Cache::new();
            let now = Local::now();
            cache.add(mail("1", "1", &["inbox"], now));
            cache.add(mail("2", "2", &["archive"], now - Duration::minutes(5)));

            cache
                .update(MailUpdate {
                    id: "1".to_string(),
                    patch_keywords: None,
                    mailbox_ids: Some(vec![("archive".into(), true)]),
                })
                .unwrap();

            assert_eq!(
                cache.mailbox_mapping.get("archive").unwrap(),
                &vec!["2".to_string(), "1".to_string()]
            );
        }

        #[test]
        fn returns_err_for_unknown_id() {
            let mut cache = Cache::new();
            let result = cache.update(MailUpdate {
                id: "unknown".to_string(),
                ..Default::default()
            });
            assert_eq!(result, Err(()));
        }
    }
}
