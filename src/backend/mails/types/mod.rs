mod mail_address;
mod mail_data;
mod mail_display;
mod mail_entry;
mod mail_keyword;
mod mail_update;

pub use mail_address::MailAddress;
pub use mail_data::{MailData, MailDataAttachment, MailDataRest};
pub use mail_display::{MailDisplay, ThreadMarker, addresses_to_string};
pub use mail_entry::MailEntry;
pub use mail_keyword::MailKeyword;
pub use mail_update::MailUpdate;

pub type MailId = String;
pub type ThreadId = String;
