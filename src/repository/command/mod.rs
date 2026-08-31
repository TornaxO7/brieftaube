mod mailbox;
pub use mailbox::*;

use crate::datasource::{Cache, Remote};

#[derive(Debug)]
pub enum Command<C, R>
where
    C: Cache,
    R: Remote,
{
    Mailbox(MailboxCommand<C, R>),
    Quit,
}
