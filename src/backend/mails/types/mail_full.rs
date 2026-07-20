use super::{MailAddress, MailKeyword, ThreadId};
use crate::backend::mailbox::types::MailboxId;
use chrono::{DateTime, Local};
use jmap_client::email::Property;
use std::collections::HashSet;

pub struct MailDataFull {
    pub id: String,
    pub thread_id: ThreadId,
    pub keywords: HashSet<MailKeyword>,
    pub from: Vec<MailAddress>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub subject: String,
    pub preview: String,
    pub received_at: DateTime<Local>,
    pub has_attachment: bool,
    pub mailbox_ids: HashSet<MailboxId>,
}

/// The rest of information to combine `MailData` and `MailDataRest` into `MailDataFull`.
pub struct MailDataRest {}

impl MailDataRest {
    pub const PROPERTIES: [Property; 3] = [
        Property::Attachments,
        Property::HtmlBody,
        Property::TextBody,
    ];
}
