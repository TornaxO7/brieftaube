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
    /// The first mail of each thread within the given mailbox.
    /// Sorted by `received_at`.
    // - Guaranteed that there's a `MailData` for each `MailId` here
    root_mails: HashMap<MailboxId, Vec<MailId>>,
    // - Guaranteed that there's a `MailData` for each `MailId` here
    // - Sorted by `received_at`
    thread_mapping: HashMap<ThreadId, Vec<MailId>>,
    state: GetState,
}

/// Methods used by the backend
impl Cache {
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
            root_mails: HashMap::new(),
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

    pub fn get_root_mails(&self, id: &MailboxId) -> Option<&[MailId]> {
        self.root_mails
            .get(id)
            .map(|root_mails| root_mails.as_slice())
    }
}

// helper methods
impl Cache {
    fn add_to_root_mails(&mut self, mail_id: &MailId, mailbox_id: MailboxId) {
        let mail = self.mails.get(mail_id).ok_or(()).unwrap();

        self.root_mails
            .entry(mailbox_id)
            .and_modify(|mailbox_mails| {
                let other_mail_in_same_thread =
                    mailbox_mails
                        .iter()
                        .enumerate()
                        .find_map(|(idx, other_id)| {
                            let other = self.mails.get(other_id).cloned().unwrap();

                            (other.thread_id == mail.thread_id).then_some((idx, other))
                        });

                match other_mail_in_same_thread {
                    Some((idx, other)) => {
                        if mail.received_at < other.received_at {
                            mailbox_mails[idx] = mail.id.clone();
                        }
                    }
                    None => {
                        let idx = match get_idx_by_received_at(&self.mails, mailbox_mails, &mail) {
                            Ok(idx) => idx,
                            Err(idx) => idx,
                        };

                        mailbox_mails.insert(idx, mail.id.clone());
                    }
                }
            })
            .or_insert(vec![mail.id.clone()]);
    }
}

// Methods altering the cache
impl Cache {
    pub fn flush(&mut self) {
        self.mails.clear();
        self.root_mails.clear();
        self.state.clear();
    }

    pub fn add(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), mail.clone());

        // add to `root_mails`
        for mailbox_id in mail.mailbox_ids.iter().cloned() {
            self.add_to_root_mails(&mail.id, mailbox_id);
        }

        // add to `thread_mapping`
        self.thread_mapping
            .entry(mail.thread_id.clone())
            .and_modify(|thread_mails| {
                let idx = match get_idx_by_received_at(&self.mails, thread_mails, &mail) {
                    Ok(idx) => idx,
                    Err(idx) => idx,
                };
                thread_mails.insert(idx, mail.id.clone());
            })
            .or_insert(vec![mail.id.clone()]);
    }

    pub fn remove(&mut self, id: MailId) -> Option<MailData> {
        let mail = self.mails.remove(&id)?;

        // remove from `mailbox_mapping`
        for mailbox_id in mail.mailbox_ids.iter() {
            let mailbox_mails = self.root_mails.get_mut(mailbox_id).unwrap();
            if let Some(idx) = mailbox_mails.iter().position(|id| id == &mail.id) {
                mailbox_mails.remove(idx);
            }
        }

        // remove from `threads_mapping`
        {
            let thread_mapping = self.thread_mapping.get_mut(&mail.thread_id).unwrap();
            if let Some(idx) = thread_mapping.iter().position(|id| id == &mail.id) {
                thread_mapping.remove(idx);
            }
        }

        Some(mail)
    }

    // Returns `Err` if there's no mail with the given id.
    pub fn update(&mut self, new: MailUpdate) -> Result<(), ()> {
        if let Some(patch_keywords) = new.patch_keywords {
            let mail = self.mails.get_mut(&new.id).ok_or(())?;

            for (keyword, set) in patch_keywords {
                if set {
                    mail.keywords.insert(keyword.clone());
                } else {
                    mail.keywords.remove(&keyword);
                }
            }
        }

        if let Some(mailbox_ids) = new.mailbox_ids {
            for (new_mailbox, set) in mailbox_ids {
                if set {
                    self.add_to_root_mails(&new.id, new_mailbox.clone());

                    let mail = self.mails.get_mut(&new.id).ok_or(())?;
                    mail.mailbox_ids.insert(new_mailbox.clone());
                } else {
                    self.root_mails
                        .entry(new_mailbox.clone())
                        .and_modify(|mailbox_mails| {
                            if let Some(idx) = mailbox_mails.iter().position(|id| id == &new.id) {
                                mailbox_mails.remove(idx);
                            }
                        });

                    let mail = self.mails.get_mut(&new.id).unwrap();
                    mail.mailbox_ids.remove(&new_mailbox);
                }
            }
        }

        Ok(())
    }
}

fn get_idx_by_received_at(
    mails: &HashMap<MailId, MailData>,
    mapping: &[MailId],
    mail: &MailData,
) -> Result<usize, usize> {
    mapping.binary_search_by(|other_id| {
        let other = mails.get(other_id).unwrap();
        other.received_at.cmp(&mail.received_at)
    })
}
