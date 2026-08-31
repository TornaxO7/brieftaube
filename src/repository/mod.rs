pub mod command;
pub mod types;

use crate::{
    datasource::{Cache, Remote},
    types::MailboxId,
};
use command::*;
use tokio::sync::{mpsc, oneshot};
use types::*;

#[derive(thiserror::Error, Debug)]
pub enum Error<C, R>
where
    C: Cache,
    R: Remote,
{
    #[error("Error from cache: {0}")]
    Cache(C::Error),

    #[error("Remote error: {0}")]
    Remote(R::Error),
}

pub struct Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    cache: C,
    remote: R,
    receiver: mpsc::Receiver<Command<C, R>>,
}

impl<C, R> Repository<C, R>
where
    C: Cache,
    R: Remote,
{
    pub fn new(cache: C, remote: R, receiver: mpsc::Receiver<Command<C, R>>) -> Self {
        Self {
            cache,
            remote,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                Command::Mailbox(cmd) => match cmd {
                    MailboxCommand::GetChildren { id, tx } => {
                        let _ = tx.send(self.mailbox_children(id).await);
                    }
                },
                Command::Quit => self.quit(),
            }
        }
    }

    fn quit(&mut self) {
        self.receiver.close();
    }
}
