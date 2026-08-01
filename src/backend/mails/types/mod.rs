mod mail_address;
mod mail_body_type;
mod mail_data;
mod mail_keyword;
mod mail_update;

pub use mail_address::*;
pub use mail_body_type::MailBodyType;
pub use mail_data::*;
pub use mail_keyword::MailKeyword;
pub use mail_update::MailUpdate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);
