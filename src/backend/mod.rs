pub mod mailbox;
pub mod mails;
pub mod threads;
pub mod types;

use crate::{
    backend::{
        mailbox::types::{MailboxData, MailboxId},
        types::CollapsedMail,
    },
    config::Config,
};
use jmap_client::client::Client;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};
use tokio::task::JoinHandle;
use tracing::{debug, error, instrument};

type GetState = String;
type QueryState = String;

pub struct Backend {
    config: Rc<Config>,

    client: Arc<Client>,

    mailboxes: Arc<mailbox::MailboxBackend>,
    mails: Arc<mails::MailsBackend>,
    threads: Arc<threads::ThreadsBackend>,

    tasks: RefCell<VecDeque<JoinHandle<()>>>,
}

/// Methods needed for `main.rs`
impl Backend {
    pub async fn new() -> Self {
        let config = Rc::new(Config::load().unwrap());

        let client = Client::new()
            .credentials((config.address.trim(), config.password.trim()))
            .follow_redirects([config.host.trim()])
            .connect(&format!("http://{}", config.host.trim()))
            .await
            .map(|client| Arc::new(client))
            .unwrap();

        let session = client.session();
        assert!(
            session
                .capabilities()
                .find(|cap| cap.as_str() == jmap_client::URI::Mail.as_ref())
                .is_some(),
            "Hold up! Your server doesn't seem to support email capabilities?! Eh... That's funny... here are the information of the session: {:#?}",
            session
        );

        Self {
            client: client.clone(),
            mailboxes: Arc::new(mailbox::MailboxBackend::new(client.clone())),
            mails: Arc::new(mails::MailsBackend::new(client.clone())),
            threads: Arc::new(threads::ThreadsBackend::new(client.clone())),
            tasks: RefCell::new(VecDeque::with_capacity(8)),
            config,
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.borrow().is_empty()
    }

    pub async fn finish_next_task(&self) {
        let _done = {
            let mut tasks = self.tasks.borrow_mut();
            match tasks.front_mut() {
                Some(task) => task.await,
                None => std::future::pending().await,
            }
        };
        self.tasks.borrow_mut().pop_front();
    }
}

/// Methods for states.
impl Backend {
    #[instrument(skip(self))]
    pub fn get_child_mailboxes(&self, parent_id: Option<MailboxId>) -> Option<Vec<MailboxId>> {
        let mailbox_backend = self.mailboxes.clone();

        self.mailboxes.get_child_mailboxes(&parent_id).or_else(|| {
            self.tasks.borrow_mut().push_back(tokio::spawn(async move {
                debug!("Requesting mailboxes");
                if let Err(err) = mailbox_backend
                    .request_mailboxes_query(parent_id.clone())
                    .await
                {
                    error!("Couldn't query mailboxes:\n{err}");
                }
            }));

            None
        })
    }

    pub fn get_collapsed_mails(&self, id: &MailboxId) -> Option<Vec<CollapsedMail>> {
        let mut collapsed_mails: Vec<CollapsedMail> =
            Vec::with_capacity(self.mailboxes.get_total_threads(id).unwrap());

        match self.mails.get_root_mails(id) {
            Some(root_mails_ids) => {
                for root_mail_id in root_mails_ids {
                    let Some(root_mail) = self.mails.get_mail(&root_mail_id) else {
                        todo!("Request mail in batch");
                    };

                    let Some(root_mail_thread) = self.threads.get_thread(&root_mail.thread_id)
                    else {
                        todo!("Request thread in batch");
                        return None;
                    };

                    let root_mail_thread_has_only_one_mail = root_mail_thread.len() == 1;
                    let entry = if root_mail_thread_has_only_one_mail {
                        CollapsedMail::SingleMail(root_mail_id)
                    } else {
                        CollapsedMail::CollapsedThread(root_mail.thread_id.clone())
                    };

                    collapsed_mails.push(entry);
                }
            }
            None => {
                todo!("Request root mails")
            }
        }

        Some(collapsed_mails)
    }
}
