pub mod error;

use crate::backend::{
    GetState, MailData, MailId, MailUpdate, threads::types::ThreadId, types::RemoteData,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Store {
    mails: HashMap<MailId, RemoteData<MailData>>,
    state: GetState,
}

/// Methods used by the backend
impl Store {
    pub fn new() -> Self {
        Self {
            mails: HashMap::new(),
            state: GetState::new(),
        }
    }

    pub fn set_state(&mut self, new_state: String) {
        self.state = new_state;
    }

    pub fn get(&mut self, id: &MailId) -> &RemoteData<MailData> {
        self.get_mut(id)
    }

    pub fn get_mut(&mut self, id: &MailId) -> &mut RemoteData<MailData> {
        self.mails
            .entry(id.clone())
            .or_insert(RemoteData::NotRequested)
    }
}

impl Store {
    pub fn add(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), RemoteData::Loaded(mail));
    }

    pub fn remove(&mut self, id: &MailId) -> Option<RemoteData<MailData>> {
        self.mails.remove(id)
    }

    /// Panics if there's no mail with the given id to be updated.
    pub fn update(&mut self, new: MailUpdate) {
        let mail = self
            .get_mut(&new.id)
            .loaded_mut()
            .expect("Mail is already fetched");

        if let Some(patch_keywords) = new.patch_keywords {
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
                    mail.mailbox_ids.insert(new_mailbox.clone());
                } else {
                    mail.mailbox_ids.remove(&new_mailbox);
                }
            }
        }
    }
}
