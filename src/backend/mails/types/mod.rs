mod mail_address;
mod mail_data;
mod mail_display;
mod mail_keyword;
mod mail_update;

pub use mail_address::MailAddress;
pub use mail_data::MailData;
pub use mail_display::MailPreview;
pub use mail_keyword::MailKeyword;
pub use mail_update::MailUpdate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MailId(pub String);
