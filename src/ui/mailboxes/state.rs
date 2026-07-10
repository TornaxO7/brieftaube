use crate::backend;
use jmap_client::mailbox::Mailbox;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct State {}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {}

    pub fn update(&mut self) {}
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
