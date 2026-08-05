mod store;

pub mod impls;
pub mod types;

use crate::backend::mails::types::MailData;
use types::MailId;

pub use store::Store;
