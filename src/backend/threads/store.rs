use crate::backend::{GetState, mails::types::MailId, threads::types::ThreadId};
use std::collections::HashMap;

pub struct Store {
    threads: HashMap<ThreadId, Vec<MailId>>,
    state: GetState,
}

impl Store {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            state: GetState::new(),
        }
    }

    pub fn get_state(&self) -> String {
        self.state.clone()
    }

    pub fn get_mails(&mut self, id: &ThreadId) -> &[MailId] {
        self.threads.get(id).unwrap()
    }

    pub fn get_mails_mut(&mut self, id: &ThreadId) -> &mut [MailId] {
        self.threads.get_mut(id).unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }
}

/// Cache alterning methods
impl Store {
    pub fn flush(&mut self) {
        self.threads.clear();
        self.state.clear();
    }

    pub fn set_state(&mut self, new_state: String) {
        self.state = new_state;
    }

    pub fn add(&mut self, thread_id: &ThreadId, mail_ids: Vec<MailId>) {
        self.threads.insert(thread_id.clone(), mail_ids);
    }

    pub fn remove(&mut self, id: &ThreadId) -> Option<Vec<MailId>> {
        self.threads.remove(id)
    }
}
