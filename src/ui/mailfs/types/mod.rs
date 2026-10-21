mod mail_entry;
mod mail_preview;
mod mailbox_entry;

pub use mail_entry::*;
pub use mail_preview::*;
pub use mailbox_entry::*;

pub trait Selectable {
    fn is_selected(&self) -> bool;
}
