pub mod error;

use super::{
    MailId,
    types::{MailEntry, ThreadId},
};
use crate::backend::{
    mailbox::types::MailboxId,
    mails::{
        MailData,
        types::{MailEntryType, MailUpdate},
    },
};
use error::UnfoldError;
use jmap_client::core::{
    query::QueryResponse,
    response::{EmailGetResponse, ThreadGetResponse},
};
use std::collections::HashMap;
use tracing::warn;

#[derive(Default)]
pub struct Cache {
    mails: HashMap<MailId, MailData>,
    // sorted by received_at
    mailbox_mapping: HashMap<MailboxId, Vec<MailEntry>>,
    thread_mapping: HashMap<ThreadId, Vec<MailId>>,

    thread_get_state: String,
    email_get_state: String,
    _query_state: String,
}

impl Cache {
    pub fn init_mailbox(
        mut query_response: QueryResponse,
        mut get_mail_response: EmailGetResponse,
        mut get_thread_response: ThreadGetResponse,
    ) -> Self {
        let raw_mail_list = get_mail_response.take_list();

        let query_state = query_response.take_query_state();
        let email_get_state = get_mail_response.take_state();
        let thread_get_state = get_thread_response.take_state();

        let thread_mapping: HashMap<ThreadId, Vec<MailId>> = get_thread_response
            .take_list()
            .into_iter()
            .map(|thread| (thread.id().to_string(), thread.email_ids().to_vec()))
            .collect();

        let mails: HashMap<MailId, MailData> = raw_mail_list
            .into_iter()
            .map(MailData::new)
            .map(|mail| (mail.id.clone(), mail))
            .collect();

        let mailbox_mapping: HashMap<MailboxId, Vec<MailEntry>> = {
            let mut idx: HashMap<MailboxId, Vec<MailEntry>> = HashMap::with_capacity(mails.len());

            for mail in mails.values() {
                for mailbox in mail.mailbox_ids.iter() {
                    idx.entry(mailbox.clone())
                        .and_modify(|mailbox_mails| {
                            let idx = mailbox_mails.partition_point(|entry| {
                                let other = mails.get(&entry.id).unwrap();
                                other.received_at > mail.received_at
                            });

                            mailbox_mails.insert(idx, MailEntry::new_root(&mail));
                        })
                        .or_insert(vec![MailEntry::new_root(&mail)]);
                }
            }

            idx
        };

        Self {
            mails,
            mailbox_mapping,
            thread_mapping,

            email_get_state,
            _query_state: query_state,
            thread_get_state,
        }
    }

    pub fn is_mailbox_initialised(&self, id: &MailboxId) -> bool {
        self.mailbox_mapping.contains_key(id)
    }

    pub fn get_mail_state(&self) -> String {
        self.email_get_state.clone()
    }

    pub fn set_mail_state(&mut self, new_state: String) {
        self.email_get_state = new_state;
    }

    pub fn get_mail(&self, id: &MailId) -> Option<&MailData> {
        self.mails.get(id)
    }

    pub fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData> {
        self.mails.get_mut(id)
    }

    pub fn get_mails_from_mailbox(&self, id: &MailboxId) -> Option<&[MailEntry]> {
        self.mailbox_mapping.get(id).map(|mails| mails.as_slice())
    }

    pub fn add_mail(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), mail.clone());

        // add to thread-list in correct order
        {
            self.thread_mapping
                .entry(mail.thread_id.clone())
                .and_modify(|thread_mails| {
                    let idx = thread_mails.partition_point(|id| {
                        let other = self.mails.get(id).unwrap();

                        other.received_at > mail.received_at
                    });

                    thread_mails.insert(idx, mail.id.clone());
                })
                .or_insert(vec![mail.id.clone()]);
        }
    }

    pub fn get_thread(&self, id: &ThreadId) -> Option<&[MailId]> {
        self.thread_mapping.get(id).map(|mails| mails.as_slice())
    }
}

// Actions
impl Cache {
    pub fn update_mail(&mut self, new: MailUpdate) {
        // update mail keywords
        if let Some(patch_keywords) = &new.patch_keywords {
            let mail = self.mails.get_mut(&new.id).unwrap();
            for (keyword, set) in patch_keywords {
                if *set {
                    mail.keywords.insert(keyword.clone());
                } else {
                    mail.keywords.remove(keyword);
                }
            }
        }

        // update mailbox mappings
        if let Some(mailbox_ids) = &new.mailbox_ids {
            for (mailbox_id, set) in mailbox_ids {
                if *set {
                    let mail = self.mails.get(&new.id).unwrap();

                    self.mailbox_mapping
                        .entry(mailbox_id.clone())
                        .and_modify(|children| {
                            let idx = children.partition_point(|entry| {
                                let other = self.mails.get(&entry.id).unwrap();
                                other.received_at > mail.received_at
                            });

                            children.insert(idx, MailEntry::new_root(&mail));
                        });

                    let mail = self.mails.get_mut(&new.id).unwrap();
                    mail.mailbox_ids.insert(mailbox_id.clone());
                } else {
                    self.mailbox_mapping
                        .entry(mailbox_id.clone())
                        .and_modify(|children| children.retain(|entry| entry.id != new.id));

                    let mail = self.mails.get_mut(&new.id).unwrap();
                    mail.mailbox_ids.remove(mailbox_id);
                }
            }
        }
    }

    pub fn fold_thread(&mut self, mailbox: &MailboxId, thread: &ThreadId) {
        let Some(mailbox_mails) = self.mailbox_mapping.get_mut(mailbox) else {
            warn!("Couldn't find mailbox of thread which should be folded.");
            return;
        };

        mailbox_mails.retain(|entry| {
            let is_in_same_thread = &entry.thread != thread;
            let is_child = entry.ty == MailEntryType::Child;

            is_child && is_in_same_thread
        });
    }

    pub fn insert_thread(
        &mut self,
        mut get_thread_response: ThreadGetResponse,
        mut get_mail_response: EmailGetResponse,
    ) {
        self.thread_get_state = get_thread_response.take_state();
        self.email_get_state = get_mail_response.take_state();

        for mail in get_mail_response.take_list() {
            self.add_mail(MailData::new(mail));
        }
    }

    pub fn unfold_mail(&mut self, mailbox: &MailboxId, id: &MailId) -> Result<bool, UnfoldError> {
        let mailbox_mails = self
            .mailbox_mapping
            .get_mut(mailbox)
            .ok_or(UnfoldError::MailboxMailsMissing)?;

        let unfold_pos = mailbox_mails
            .iter()
            .position(|entry| &entry.id == id)
            .expect("The given mail doesn't belong to the mailbox?! Where did that come from?!")
            + 1;

        let mail = self
            .mails
            .get(id)
            .expect("Can't unfold mail: Couldn't find mail to unfold. Where did that mail id come from??? <.<");

        {
            let already_unfolded = match mailbox_mails.get(unfold_pos) {
                Some(entry) => {
                    let is_child = entry.ty == MailEntryType::Child;
                    let is_in_same_thread = entry.thread == mail.thread_id;
                    is_child && is_in_same_thread
                }
                None => false,
            };

            if already_unfolded {
                return Ok(false);
            }
        }

        let thread_mails = self
            .thread_mapping
            .get(&mail.thread_id)
            .ok_or(UnfoldError::MissingThreadMails(mail.thread_id.clone()))?;

        mailbox_mails.splice(
            unfold_pos..unfold_pos,
            thread_mails.iter().skip(1).map(|thread_mail| MailEntry {
                id: thread_mail.clone(),
                thread: mail.thread_id.clone(),
                ty: MailEntryType::Child,
            }),
        );

        return Ok(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Local, TimeZone};
    use std::collections::HashSet;

    #[test]
    fn add_mail() {
        let mut cache = Cache::default();

        let mail1 = MailData {
            id: "1".into(),
            thread_id: "1".into(),
            mailbox_ids: HashSet::from(["1".into()]),
            received_at: Local.timestamp_opt(10, 0).unwrap(),
            ..Default::default()
        };

        let mail2 = MailData {
            id: "2".into(),
            thread_id: "1".into(),
            mailbox_ids: HashSet::from(["1".into()]),
            received_at: Local.timestamp_opt(5, 0).unwrap(),
            ..Default::default()
        };

        let mail3 = MailData {
            id: "3".into(),
            thread_id: "1".into(),
            mailbox_ids: HashSet::from(["1".into()]),
            received_at: Local.timestamp_opt(15, 0).unwrap(),
            ..Default::default()
        };

        cache.add_mail(mail1.clone());
        cache.add_mail(mail2.clone());
        cache.add_mail(mail3.clone());

        // check mails
        assert_eq!(&mail1, cache.mails.get(&mail1.id).unwrap());
        assert_eq!(&mail2, cache.mails.get(&mail2.id).unwrap());
        assert_eq!(&mail3, cache.mails.get(&mail3.id).unwrap());

        // check mailbox
        assert_eq!(
            [
                // all are in the same thread => Don't display all of them unless `unfold` is called
                MailEntry::new_root(&mail2),
            ]
            .as_slice(),
            cache.mailbox_mapping.get("1").unwrap()
        );

        // check threads
        assert_eq!(
            [mail2.id.clone(), mail1.id.clone(), mail3.id.clone()].as_slice(),
            cache.thread_mapping.get("1").unwrap()
        );
    }
}
