mod root_mails;
mod threads;

use crate::{
    backend::{
        Account,
        mailboxes::mailbox::{root_mails::RootMails, threads::ThreadCtx},
    },
    ui::MailboxId,
};
use jmap_client::mailbox::Mailbox;
use std::{collections::HashMap, thread::ThreadId};

pub struct MailboxCtx {
    inner: Mailbox,

    root_mails: Option<RootMails>,
    threads: HashMap<ThreadId, ThreadCtx>,
}

impl MailboxCtx {
    pub fn new(inner: Mailbox) -> Self {
        Self {
            inner,
            root_mails: None,
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
    pub fn init_root_mails(&self, id: MailboxId) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let is_not_initialised = {
                let data = data.lock().unwrap();
                let mailboxes = data.mailboxes.as_ref().expect("Mailboxes is intialised");

                mailboxes.get_mailbox(&id).root_mails.is_none()
            };

            if is_not_initialised {
                let root_mails = RootMails::new(&client, id.clone()).await;

                let mut data = data.lock().unwrap();
                data.mailboxes
                    .as_mut()
                    .unwrap()
                    .get_mut_mailbox(&id)
                    .root_mails = Some(root_mails);
            }
        });
    }
}
