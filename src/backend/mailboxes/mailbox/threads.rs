use jmap_client::{client::Client, email::Email};

use crate::ui::ThreadId;

const INIT_AMOUNT_THREADS: usize = 10;

pub struct ThreadCtx {
    state: String,
    root_mails: Vec<Email>,
}

impl ThreadCtx {
    pub async fn new(client: &Client, id: ThreadId) -> Self {
        let mut request = client.build();

        request.get_thread().ids(Some([id]));
        let mut response = request.send_get_thread().await.unwrap();

        let state = response.take_state();
        let root_mails = {
            let mut request = client.build();
            request
                .get_email()
                .ids(Some(response.list()[0].email_ids()));
            let mut response = request.send_get_email().await.unwrap();

            response.take_list()
        };

        Self { state, root_mails }
    }
}
