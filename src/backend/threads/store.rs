use crate::backend::{GetState, mails::types::MailId, threads::types::ThreadId, types::RemoteData};
use std::collections::HashMap;

pub struct Store {
    threads: HashMap<ThreadId, RemoteData<Vec<MailId>>>,
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

    pub fn get_mail_ids(&mut self, id: &ThreadId) -> &RemoteData<Vec<MailId>> {
        self.threads
            .entry(id.clone())
            .or_insert(RemoteData::NotRequested)
    }

    pub fn get_mail_ids_mut(&mut self, id: &ThreadId) -> &mut RemoteData<Vec<MailId>> {
        self.threads
            .entry(id.clone())
            .or_insert(RemoteData::NotRequested)
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
        self.threads
            .insert(thread_id.clone(), RemoteData::Loaded(mail_ids));
    }

    pub fn remove(&mut self, id: &ThreadId) -> Option<RemoteData<Vec<MailId>>> {
        self.threads.remove(id)
    }
}
