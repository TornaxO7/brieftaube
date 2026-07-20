use super::{MailId, MailKeyword};
use crate::backend::mailbox::types::MailboxId;

#[derive(Debug, Clone, Default)]
pub struct MailUpdate {
    pub id: MailId,
    pub patch_keywords: Option<Vec<(MailKeyword, bool)>>,
    pub mailbox_ids: Option<Vec<(MailboxId, bool)>>,
}
