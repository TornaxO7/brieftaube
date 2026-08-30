use std::collections::HashMap;

use super::{MailId, MailKeyword, MailboxId};

#[derive(Debug, Clone, Default)]
pub struct MailUpdate {
    pub id: MailId,
    pub patch_keywords: Option<HashMap<MailKeyword, bool>>,
    pub mailbox_ids: Option<HashMap<MailboxId, bool>>,
}

impl MailUpdate {
    /// Returns true if there are no updates
    pub fn is_empty(&self) -> bool {
        self.patch_keywords.is_none() && self.mailbox_ids.is_none()
    }
}
