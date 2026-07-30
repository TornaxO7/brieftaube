mod mail_address;
mod mail_data;
mod mail_keyword;
mod mail_update;

pub use mail_address::{MailAddress, addresses_to_string};
pub use mail_data::{MailData, MailDataRest};
pub use mail_keyword::MailKeyword;
pub use mail_update::MailUpdate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);
