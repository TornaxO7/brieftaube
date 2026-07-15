use crate::{backend::Account, utils::ui::ThreadId};
use jmap_client::{client::Client, email::Email};
use tracing::trace;

pub struct Thread {
    mails: Vec<Email>,

    thread_state: String,
    _mails_state: String,
}

impl Thread {
    pub async fn new(client: &Client, id: ThreadId) -> color_eyre::Result<Self> {
        let mut request = client.build();

        request.get_thread().ids(Some([id]));
        let mut response = request.send_get_thread().await?;
        let thread_state = response.take_state();

        let (mails, mails_state) = {
            let mut request = client.build();
            request
                .get_email()
                .ids(Some(response.list()[0].email_ids()))
                .arguments()
                .fetch_all_body_values(true);
            let mut response = request.send_get_email().await?;

            (response.take_list(), response.take_state())
        };

        Ok(Self {
            mails,

            thread_state,
            _mails_state: mails_state,
        })
    }
}

impl Account {
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
            Err(_is_locked) => None,
        }
    }
}

impl Account {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn init_thread(&self, id: ThreadId) {
        let data = self.data.clone();
        let client = self.client.clone();

        trace!("Init thread: {}", id);
        self.tasks.lock().unwrap().spawn(async move {
            let mut data = data.lock().await;
            let threads = &mut data.threads;

            let is_not_initialised = !threads.contains_key(&id);
            if is_not_initialised {
                let thread = Thread::new(&client, id.clone()).await?;
                trace!("Fetched thread '{}' successfully.", id);

                threads.insert(id, thread);
            }
            Ok(())
        });
    }
}
