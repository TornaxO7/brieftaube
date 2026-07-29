pub mod error;

use super::MailId;
use crate::backend::{
    GetState,
    mails::{MailData, types::MailUpdate},
    threads::types::ThreadId,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Cache {
    mails: HashMap<MailId, MailData>,
    state: GetState,
}

/// Methods used by the backend
impl Cache {
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
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
}

// Methods altering the cache
impl Cache {
    pub fn flush(&mut self) {
        self.mails.clear();
        self.state.clear();
    }

    pub fn add(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), mail.clone());
    }

    pub fn remove(&mut self, id: MailId) -> Option<MailData> {
        self.mails.remove(&id)
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
            let mail = self.mails.get_mut(&new.id).ok_or(())?;

            for (new_mailbox, set) in mailbox_ids {
                if set {
                    mail.mailbox_ids.insert(new_mailbox.clone());
                } else {
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
