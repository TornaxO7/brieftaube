use crate::{datasource::Remote, types::AccountId};
use jmap_client::client::Client;
use std::sync::Arc;

pub struct JmapAccount {
    client: Arc<Client>,
    account_id: AccountId,
}

impl JmapAccount {
    pub fn new(client: Arc<Client>, account_id: AccountId) -> Self {
        Self { client, account_id }
    }
}
