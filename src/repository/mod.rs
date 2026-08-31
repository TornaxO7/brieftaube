pub mod mail;
pub mod mailbox;
pub mod thread;

use crate::datasource::{Cache, Remote};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    Mail(mail::Command<C, R>),
    Mailbox(mailbox::Command<C, R>),
    Thread(thread::Command<C, R>),
    Quit,
}

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
    // TODO: Put it inside `RwLock`
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
                Command::Mail(cmd) => match cmd {
                    mail::Command::QueryRootMails {
                        mailbox,
                        start,
                        limit,
                        tx,
                    } => {
                        let _ = tx.send(self.query_root_mails(mailbox, start, limit).await);
                    }
                    mail::Command::GetTextBody { id, tx } => {
                        let _ = tx.send(self.get_mail_text_body(id).await);
                    }
                    mail::Command::GetHtmlBody { id, tx } => {
                        let _ = tx.send(self.get_mail_html_body(id).await);
                    }
                    mail::Command::GetAttachments { id, tx } => {
                        let _ = tx.send(self.get_mail_attachments(id).await);
                    }
                },
                Command::Mailbox(cmd) => match cmd {
                    mailbox::Command::GetChildren { id, tx } => {
                        let _ = tx.send(self.get_mailbox_children(id).await);
                    }
                },
                Command::Thread(cmd) => match cmd {
                    thread::Command::GetThread { id, tx } => {
                        let _ = tx.send(self.get_thread(id).await);
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
