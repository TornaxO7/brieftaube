use super::MailKeyword;
use crate::types::{MailAddresses, MailboxId, MessageId};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailNew {
    pub mailbox_ids: HashSet<MailboxId>,
    pub keywords: HashSet<MailKeyword>,
    pub from: Option<MailAddresses>,
    pub to: Option<MailAddresses>,
    pub cc: Option<MailAddresses>,
    pub bcc: Option<MailAddresses>,
    pub subject: Option<String>,
    pub in_reply_to: Option<Vec<MessageId>>,
    pub references: Option<Vec<MessageId>>,
}
