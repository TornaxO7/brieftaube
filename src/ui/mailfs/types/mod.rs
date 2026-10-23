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

    fn set_selection(&mut self, selected: bool);

    fn toggle_selection(&mut self) {
        self.set_selection(!self.is_selected());
    }
}

impl<T: Selectable> Selectable for Loadable<T> {
    fn is_selected(&self) -> bool {
        match self {
            Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => false,
            Loadable::Loaded(value) => value.is_selected(),
        }
    }

    fn set_selection(&mut self, selected: bool) {
        if let Loadable::Loaded(value) = self {
            value.set_selection(selected);
        }
    }
}
