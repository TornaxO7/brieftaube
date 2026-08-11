use super::{MailboxId, SortOrder};
use jmap_client::{
    Set,
    core::set::SetRequest,
    mailbox::{Mailbox, Role},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxUpdate {
    pub id: MailboxId,
    pub name: Option<String>,
    pub role: Option<Role>,
    pub sort_order: Option<SortOrder>,
    pub parent_id: Option<Option<MailboxId>>,
}

impl MailboxUpdate {
    pub fn set_request(&self, request: &mut SetRequest<Mailbox<Set>>) {
        let update = request.update(&self.id);
        if let Some(name) = &self.name {
            update.name(name);
        }

        if let Some(role) = self.role.clone() {
            update.role(role);
        }

        if let Some(sort_order) = self.sort_order.clone() {
            update.sort_order(sort_order);
        }

        if let Some(parent_id) = self.parent_id.clone() {
            update.parent_id(parent_id);
        }
    }
}
