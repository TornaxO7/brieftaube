use crate::utils::ui::MailboxId;
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

    pub fn mails(&self) -> &[Email] {
        self.mails.as_slice()
    }
}
