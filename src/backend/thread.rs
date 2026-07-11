use crate::{backend::Account, ui::ThreadId};
use jmap_client::{client::Client, email::Email};

pub struct Thread {
    mails: Vec<Email>,

    thread_state: String,
    _mails_state: String,
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
                .ids(Some(response.list()[0].email_ids()))
                .arguments()
                .fetch_all_body_values(true);
            let mut response = request.send_get_email().await.unwrap();

            (response.take_list(), response.take_state())
        };

        Self {
            mails,

            thread_state,
            _mails_state: mails_state,
        }
    }
}

impl Account {
    pub fn init_thread(&self, id: ThreadId) {
        let data = self.data.clone();
        let client = self.client.clone();

        self.tasks.lock().unwrap().spawn(async move {
            let is_not_initialised = {
                let data = data.lock().unwrap();
                !data.threads.contains_key(&id)
            };

            if is_not_initialised {
                let thread = Thread::new(&client, id.clone()).await;

                let mut data = data.lock().unwrap();
                data.threads.insert(id, thread);
            }
        });
    }

    pub fn get_thread_mails(&self, id: &ThreadId, state: &str) -> Option<(Vec<Email>, String)> {
        match self.data.try_lock() {
            Ok(data) => {
                let Some(thread) = data.threads.get(id) else {
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
