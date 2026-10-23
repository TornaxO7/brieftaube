mod accounts;
mod mailbox;
mod thread;
mod users;

pub use accounts::*;
pub use mailbox::*;
pub use thread::*;
pub use users::*;

pub trait Column {
    fn navigate_up(&mut self, offset: u16);

    fn navigate_down(&mut self, offset: u16);

    fn navigate_to_bottom(&mut self);

    fn navigate_to_top(&mut self);

    fn len(&self) -> usize;
}
