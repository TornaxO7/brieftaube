use crate::{
    backend::Account,
    ui::{MailboxId, ThreadId},
};
use jmap_client::{client::Client, email::Email};

const INIT_AMOUNT_THREADS: usize = 10;

pub struct Thread {
    mails: Vec<Email>,

    thread_state: String,
    mails_state: String,
}

impl Thread {
    pub async fn new(client: &Client, id: ThreadId) -> Self {
        let mut request = client.build();

        request.get_thread().ids(Some([id]));
        let mut response = request.send_get_thread().await.unwrap();
        let thread_state = response.take_state();

        let (mails, mails_state) = {
            let mut request = client.build();
            request
                .get_email()
                .ids(Some(response.list()[0].email_ids()));
            let mut response = request.send_get_email().await.unwrap();

            (response.take_list(), response.take_state())
        };

        Self {
            mails,

            thread_state,
            mails_state,
        }
    }
}

impl Account {
    pub fn get_thread_mails(
        &self,
        mailbox_id: &MailboxId,
        thread_id: &ThreadId,
        state: &str,
    ) -> Option<(Vec<Email>, String)> {
        match self.data.try_lock() {
            Ok(data) => {
                let Some(mailboxes) = data.mailboxes.as_ref() else {
                    return None;
                };

                let mailbox = mailboxes.get_mailbox(mailbox_id);
                let Some(thread) = mailbox.threads.get(thread_id) else {
                    return None;
                };

                let has_changed = thread.thread_state != state;
                if has_changed {
                    let mails = thread.mails.clone();
                    Some((mails, thread.thread_state.clone()))
                } else {
                    None
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => None,
            Err(std::sync::TryLockError::Poisoned(err)) => unreachable!("{:?}", err),
        }
    }
}
