use crate::backend::{mails::types::MailId, threads::types::ThreadId};
use std::collections::HashMap;

pub struct Cache {
    threads: HashMap<ThreadId, Vec<MailId>>,
    state: String,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            state: String::new(),
        }
    }

    pub fn get_state(&self) -> String {
        self.state.clone()
    }

    pub fn get_thread_mails(&self, id: &ThreadId) -> Option<&[MailId]> {
        self.threads.get(id).map(|mail_ids| mail_ids.as_slice())
    }

    pub fn is_empty(&self) -> bool {
        self.threads.is_empty()
    }
}

/// Cache alterning methods
impl Cache {
    pub fn flush(&mut self) {
        self.threads.clear();
        self.state.clear();
    }

    pub fn set_state(&mut self, new_state: String) {
        self.state = new_state;
    }

    pub fn insert(&mut self, thread_id: ThreadId, mail_ids: Vec<MailId>) -> Option<Vec<MailId>> {
        self.threads.insert(thread_id, mail_ids)
    }
}
