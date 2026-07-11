use crate::{backend::Account, utils::ui::MailboxId};
use jmap_client::{client::Client, email::Email};

const INIT_ROOT_MAILS: usize = 10;

pub struct RootMails {
    mails: Vec<Email>,
    state: String,
}

impl RootMails {
    pub async fn new(client: &Client, id: MailboxId) -> Self {
        let mut request = client.build();
        request
            .query_email()
            .filter(jmap_client::email::query::Filter::InMailbox { value: id })
            .sort([jmap_client::email::query::Comparator::received_at().descending()])
            .arguments()
            .collapse_threads(true);
        let mut response = request.send_query_email().await.unwrap();

        let state = response.take_query_state();

        let mails = {
            let mut request = client.build();
            request.get_email().ids(Some(response.ids()));
            let mut response = request.send_get_email().await.unwrap();

            response.take_list()
        };

        Self { mails, state }
    }
}

impl Account {
    pub fn get_root_mails(&self, id: &MailboxId, state: &str) -> Option<(Vec<Email>, String)> {
        match self.data.try_lock() {
            Ok(data) => {
                let Some(mailboxes) = data.mailboxes.as_ref() else {
                    return None;
                };

                let mailbox = mailboxes.get_mailbox(id);
                let Some(root_mails) = mailbox.root_mails.as_ref() else {
                    return None;
                };

                let has_changed = state != root_mails.state;
                if has_changed {
                    Some((root_mails.mails.clone(), root_mails.state.clone()))
                } else {
                    None
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => None,
            Err(std::sync::TryLockError::Poisoned(err)) => unreachable!("{:?}", err),
        }
    }
}
