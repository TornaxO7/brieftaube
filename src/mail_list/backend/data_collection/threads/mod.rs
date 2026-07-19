mod thread;

use super::{Data, types::MailData};
use crate::utils::{MailId, ThreadId};
use std::collections::{HashMap, hash_map::Entry};
pub use thread::Thread;

pub struct Threads {
    threads: HashMap<ThreadId, Thread>,
    mail_index: HashMap<MailId, ThreadId>,
}

impl Threads {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            mail_index: HashMap::new(),
        }
    }

    pub fn get_thread(&self, id: &ThreadId) -> Option<&Thread> {
        self.threads.get(id)
    }

    pub fn get_thread_entry<'a>(&'a mut self, id: ThreadId) -> Entry<'a, String, Thread> {
        self.threads.entry(id)
    }

    pub fn get_mail_from_thread(
        &self,
        thread_id: &ThreadId,
        mail_id: &MailId,
    ) -> Option<&MailData> {
        self.threads
            .get(thread_id)
            .and_then(|thread| thread.get_mail(mail_id))
    }
}

impl Data for Threads {
    fn get_mail(&self, id: &MailId) -> Option<&MailData> {
        self.mail_index
            .get(id)
            .and_then(|thread_id| self.threads.get(thread_id))
            .and_then(|thread| thread.get_mail(id))
    }

    fn get_mail_mut(&mut self, id: &MailId) -> Option<&mut MailData> {
        self.mail_index
            .get_mut(id)
            .and_then(|thread_id| self.threads.get_mut(thread_id))
            .and_then(|thread| thread.get_mail_mut(id))
    }
}
