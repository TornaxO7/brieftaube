mod account;
mod blob_id;
mod identity;
mod mail;
mod mailbox;
mod thread_id;

pub use account::*;
pub use blob_id::*;
pub use identity::*;
pub use mail::*;
pub use mailbox::*;
pub use thread_id::*;

const INTERN_ID_PREFIX: &str = "brieftaube-";
