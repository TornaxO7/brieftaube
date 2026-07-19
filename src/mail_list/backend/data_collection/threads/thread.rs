use crate::{
    mail_list::backend::data_collection::{Data, types::MailData},
    utils::MailId,
};
use std::collections::HashMap;

type ThreadState = String;

#[derive(Debug, Clone)]
pub struct Thread {
    mails: Vec<MailData>,
    mapping: HashMap<MailId, usize>,

    state: ThreadState,
}

impl Thread {
    pub fn new(state: ThreadState, mails: Vec<MailData>) -> Self {
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

    pub fn mails(&self) -> &[MailData] {
        &self.mails
    }
}

impl Data for Thread {
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
