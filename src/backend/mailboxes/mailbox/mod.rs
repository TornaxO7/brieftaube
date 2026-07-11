mod threads;

use crate::{
    backend::{Account, mailboxes::mailbox::threads::Thread},
    ui::{MailboxId, ThreadId},
};
use jmap_client::mailbox::Mailbox;
use std::collections::HashMap;

pub struct MailboxCtx {
    inner: Mailbox,

    threads: HashMap<ThreadId, Thread>,
}

impl MailboxCtx {
    pub fn new(inner: Mailbox) -> Self {
        Self {
            inner,
            threads: HashMap::default(),
        }
    }

    pub fn id(&self) -> &str {
        self.inner.id().unwrap()
    }

    pub fn mailbox(&self) -> &Mailbox {
        &self.inner
    }
}

impl Account {
    pub fn init_thread(&self, mailbox_id: MailboxId, thread_id: ThreadId) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let is_not_initialised = {
                let data = data.lock().unwrap();
                let mailboxes = data.mailboxes.as_ref().unwrap();
                let mailbox = mailboxes.get_mailbox(&mailbox_id);

                mailbox.threads.get(&thread_id).is_none()
            };

            if is_not_initialised {
                let thread = Thread::new(&client, thread_id.clone()).await;

                let mut data = data.lock().unwrap();
                data.mailboxes
                    .as_mut()
                    .unwrap()
                    .get_mut_mailbox(&mailbox_id)
                    .threads
                    .insert(thread_id, thread);
            }
        });
    }
}
