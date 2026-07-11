use crate::{backend::Account, ui::MailboxId};
use jmap_client::{client::Client, email::Email};

const INIT_ROOT_MAILS: usize = 10;

#[derive(Default)]
pub struct RootMails {
    mails: Vec<Email>,
    state: String,
}

impl RootMails {
    async fn new(client: &Client, id: MailboxId) -> Self {
        let mut request = client.build();
        request
            .query_email()
            .filter(jmap_client::email::query::Filter::InMailbox { value: id.clone() })
            .sort([jmap_client::email::query::Comparator::received_at().descending()])
            .limit(INIT_ROOT_MAILS)
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
    pub fn init_root_mails(&self, id: MailboxId) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let is_not_initialised = {
                let data = data.lock().unwrap();
                !data.root_mails.contains_key(&id)
            };

            if is_not_initialised {
                let root_mails = RootMails::new(&client, id.clone()).await;

                let mut data = data.lock().unwrap();
                data.root_mails.insert(id, root_mails);
            }
        });
    }

    pub fn get_root_mails(&self, id: &MailboxId, state: &str) -> Option<(Vec<Email>, String)> {
        match self.data.try_lock() {
            Ok(data) => {
                let Some(root_mails) = data.root_mails.get(id) else {
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
