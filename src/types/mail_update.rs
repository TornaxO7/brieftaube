use super::{MailId, MailKeyword};
use crate::backend::mailbox::types::MailboxId;

#[derive(Debug, Clone, Default)]
pub struct MailUpdate {
    pub id: MailId,
    // TODO: Replace `Vec` with `HashSet`?
    pub patch_keywords: Option<Vec<(MailKeyword, bool)>>,
    pub mailbox_ids: Option<Vec<(MailboxId, bool)>>,
}

impl MailUpdate {
    /// Returns true if there are no updates
    pub fn is_empty(&self) -> bool {
        self.patch_keywords.is_none() && self.mailbox_ids.is_none()
    }
}
