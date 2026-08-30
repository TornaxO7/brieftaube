use super::{MailKeyword, MailboxId};
use crate::types::{MailAddress, MessageId};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailNew {
    pub mailbox_ids: HashSet<MailboxId>,
    pub keywords: HashSet<MailKeyword>,
    pub from: Option<Vec<MailAddress>>,
    pub to: Option<Vec<MailAddress>>,
    pub cc: Option<Vec<MailAddress>>,
    pub bcc: Option<Vec<MailAddress>>,
    pub subject: Option<String>,
    pub in_reply_to: Option<Vec<MessageId>>,
    pub references: Option<Vec<MessageId>>,
}
