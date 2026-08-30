use super::{MailKeyword, MailboxId};
use jmap_client::email::{Header, HeaderValue};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct MailNew {
    pub mailbox_ids: Vec<MailboxId>,
    pub keywords: Option<HashSet<MailKeyword>>,

    pub headers: HashSet<(Header, HeaderValue)>,
}
