mod thread;

use super::{Data, types::MailData};
use crate::utils::{MailId, ThreadId};
use std::collections::HashMap;
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

    pub fn insert_thread(&mut self, id: ThreadId, thread: Thread) {
        let new = thread.clone();

        if let Some(old) = self.threads.insert(id.clone(), thread) {
            for mail in old.mails() {
                self.mail_index.remove(&mail.id);
            }
        }

        for mail in new.mails() {
            self.mail_index.insert(mail.id.clone(), id.clone());
        }
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
