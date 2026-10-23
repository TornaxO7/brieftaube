mod account_entry;
mod mail_entry;
mod mail_preview;
mod mailbox_entry;
mod user_entry;

pub mod column;

pub use account_entry::*;
pub use mail_entry::*;
pub use mail_preview::*;
pub use mailbox_entry::*;
pub use user_entry::*;

use crate::ui::Loadable;

pub trait Selectable {
    fn is_selected(&self) -> bool;
}

impl<T: Selectable> Selectable for Loadable<T> {
    fn is_selected(&self) -> bool {
        match self {
            Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => false,
            Loadable::Loaded(value) => value.is_selected(),
        }
    }
}
