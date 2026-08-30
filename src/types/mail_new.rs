use super::{MailKeyword, MailboxId};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailNew {
    pub mailbox_ids: Vec<MailboxId>,
    pub keywords: Option<HashSet<MailKeyword>>,
}
