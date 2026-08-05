mod mail_address;
mod mail_body_type;
mod mail_data;
mod mail_keyword;
mod mail_update;

pub use mail_address::*;
pub use mail_body_type::*;
pub use mail_data::*;
pub use mail_keyword::*;
pub use mail_update::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);

impl From<MailId> for String {
    fn from(id: MailId) -> Self {
        id.0.clone()
    }
}

impl From<&MailId> for String {
    fn from(id: &MailId) -> Self {
        id.0.clone()
    }
}
