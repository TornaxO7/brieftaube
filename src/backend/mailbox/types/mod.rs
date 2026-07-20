mod children;
mod entry;
mod mailbox_data;
mod mailbox_new;
mod mailbox_update;
mod mailbox_validate;

pub use children::Children;
pub use entry::Entry;
pub use mailbox_data::MailboxData;
pub use mailbox_new::MailboxNew;
pub use mailbox_update::MailboxUpdate;
pub use mailbox_validate::MailboxValidate;

pub type MailboxId = String;
pub type SortOrder = u32;
