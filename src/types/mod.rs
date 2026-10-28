mod account;
mod blob;
mod identity;
mod mail;
mod mailbox;
mod thread_id;

pub use account::*;
pub use blob::*;
pub use identity::*;
pub use mail::*;
pub use mailbox::*;
pub use thread_id::*;

const INTERN_ID_PREFIX: &str = "brieftaube-";
