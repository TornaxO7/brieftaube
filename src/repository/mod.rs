pub mod types;

use crate::{
    datasource::{Cache, Remote},
    types::MailboxId,
};
use tokio::sync::{mpsc, oneshot};
use types::*;

#[derive(Debug)]
pub enum Command {
    MailboxWindow {
        id: MailboxId,
        start: i32,
        length: u32,
        tx: oneshot::Sender<Vec<MailboxChild>>,
    },
    Quit,
}

pub struct Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    cache: C,
    remote: R,
    receiver: mpsc::Receiver<Command>,
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub fn new(cache: C, remote: R, receiver: mpsc::Receiver<Command>) -> Self {
        Self {
            cache,
            remote,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::Quit => self.quit(),
            }
        }
    }

    fn quit(&mut self) {
        self.receiver.close();
    }
}
