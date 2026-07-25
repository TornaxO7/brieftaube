mod children;
mod mailbox_data;
mod mailbox_new;
mod mailbox_update;
mod mailbox_validate;

pub use children::Children;
pub use mailbox_data::MailboxData;
pub use mailbox_new::MailboxNew;
pub use mailbox_update::MailboxUpdate;
pub use mailbox_validate::MailboxValidate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailboxId(pub String);
pub type SortOrder = u32;
