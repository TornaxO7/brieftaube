pub mod error;

use crate::backend::{
    GetState, MailData, MailId,
    mails::types::{MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailUpdate},
    threads::types::ThreadId,
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Store {
    mails: HashMap<MailId, MailData>,
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

    pub fn is_empty(&self) -> bool {
        self.mails.is_empty()
    }

    pub fn get_state(&self) -> String {
        self.state.clone()
    }

    pub fn set_state(&mut self, new_state: String) {
        self.state = new_state;
    }

    pub fn get(&self, id: &MailId) -> Option<&MailData> {
        self.mails.get(id)
    }

    pub fn set_attachments(&mut self, id: &MailId, attachments: Vec<MailDataAttachment>) {
        let mail = self.mails.get_mut(id).unwrap();
        mail.attachments = Some(attachments);
    }

    pub fn set_text_body(&mut self, id: &MailId, body: MailDataTextBody) {
        let mail = self.mails.get_mut(id).unwrap();
        mail.text_body = Some(body);
    }

    pub fn set_html_body(&mut self, id: &MailId, body: MailDataHtmlBody) {
        let mail = self.mails.get_mut(id).unwrap();
        mail.html_body = Some(body);
    }
}

impl Store {
    pub fn flush(&mut self) {
        self.mails.clear();
        self.state.clear();
    }

    pub fn add(&mut self, mail: MailData) {
        self.mails.insert(mail.id.clone(), mail.clone());
    }

    pub fn remove(&mut self, id: &MailId) -> Option<MailData> {
        self.mails.remove(id)
    }

    pub fn update(&mut self, new: MailUpdate) {
        if let Some(patch_keywords) = new.patch_keywords {
            let mail = self.mails.get_mut(&new.id).unwrap();

            for (keyword, set) in patch_keywords {
                if set {
                    mail.keywords.insert(keyword.clone());
                } else {
                    mail.keywords.remove(&keyword);
                }
            }
        }

        if let Some(mailbox_ids) = new.mailbox_ids {
            let mail = self.mails.get_mut(&new.id).unwrap();

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
