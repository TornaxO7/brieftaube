use super::{Data, types::MailData};
use crate::utils::MailId;
use jmap_client::email::Email;
use std::collections::HashMap;

type EmailQueryState = String;

pub struct RootMails {
    mails: Vec<MailData>,
    mapping: HashMap<MailId, usize>,

    state: EmailQueryState,
}

impl RootMails {
    pub fn new(mail_get_mails: Vec<Email>, state: EmailQueryState) -> Self {
        let mails: Vec<MailData> = mail_get_mails.into_iter().map(MailData::from).collect();

        let mapping = mails
            .iter()
            .enumerate()
            .map(|(idx, mail)| (mail.id.clone(), idx))
            .collect();

        Self {
            mails,
            mapping,
            state,
        }
    }
}

impl Data for RootMails {
    fn get_mail(&self, id: &MailId) -> Option<&MailData> {
        self.mapping
            .get(id)
            .cloned()
            .and_then(|idx| self.mails.get(idx))
    }

    fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData> {
        self.mapping
            .get(id)
            .cloned()
            .and_then(|idx| self.mails.get_mut(idx))
    }
}
