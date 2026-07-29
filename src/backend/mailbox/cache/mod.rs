use jmap_client::core::query::QueryResponse;
use tracing::{debug, instrument};

use super::types::{MailboxData, MailboxId};
use crate::backend::{
    GetState, QueryState,
    mailbox::types::{MailboxUpdate, ParentMailboxId},
    mails::types::MailId,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct RootMails {
    pub ids: Vec<MailId>,
    pub state: QueryState,
}

impl RootMails {
    pub fn new(mut response: QueryResponse) -> Self {
        let state = response.take_query_state();
        let ids = response.take_ids().into_iter().map(MailId).collect();

        Self { ids, state }
    }
}

// IDEA: Use `Arc` insead of the actual data for cheap clones out of the cache
pub struct Cache {
    mailboxes: HashMap<MailboxId, Arc<MailboxData>>,
    // Children always exist in `mailboxes`.
    children_mapping: HashMap<ParentMailboxId, Vec<MailboxId>>,

    /// stores the query state of the child-mailboxes of the given parent-mailbox
    children_query_state: HashMap<ParentMailboxId, QueryState>,
    /// stores the query state for querying the root mails of a mailbox
    root_mails_state: HashMap<MailboxId, RootMails>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            children_mapping: HashMap::new(),
            children_query_state: HashMap::new(),
            root_mails_state: HashMap::new(),
        }
    }

    pub fn is_initialised(&self, parent: &ParentMailboxId) -> bool {
        self.get_children_query_state(parent).is_some()
    }

    pub fn get_children_query_state(&self, parent: &ParentMailboxId) -> Option<GetState> {
        self.children_query_state.get(parent).cloned()
    }

    pub fn set_children_query_state(&mut self, parent: ParentMailboxId, new_state: GetState) {
        self.children_query_state.insert(parent, new_state);
    }

    pub fn get_root_mails(&self, parent: &MailboxId) -> Option<&RootMails> {
        self.root_mails_state.get(parent)
    }

    pub fn set_root_mails(&mut self, parent: MailboxId, root_mails: RootMails) {
        self.root_mails_state.insert(parent, root_mails);
    }

    pub fn get_data(&self, id: &MailboxId) -> Option<Arc<MailboxData>> {
        self.mailboxes.get(id).cloned()
    }

    #[instrument(skip(self))]
    pub fn get_children(&self, parent: &ParentMailboxId) -> Option<&[MailboxId]> {
        let children = self
            .children_mapping
            .get(parent)
            .map(|children| children.as_slice());

        // debug!("Children of '{parent:?}':\n{children:?}");

        children
    }

    pub fn get_children_data(
        &self,
        parent_id: &Option<MailboxId>,
    ) -> Option<Vec<Arc<MailboxData>>> {
        let children_ids = self.children_mapping.get(parent_id)?;

        children_ids
            .iter()
            .map(|id| self.mailboxes.get(id).cloned())
            .collect()
    }

    pub fn contains_mailbox_name(&self, parent: &ParentMailboxId, name: &str) -> bool {
        let Some(children) = self.children_mapping.get(parent) else {
            return false;
        };

        for child_id in children {
            let child = self.mailboxes.get(child_id).unwrap();

            if child.name == name {
                return true;
            }
        }

        false
    }

    /// Depth of the given parent (root = 0, its children = 1, ...).
    pub fn depth_of(&self, parent_id: &Option<MailboxId>) -> usize {
        let mut depth = 0;
        let mut current = parent_id.clone();

        while let Some(id) = current {
            depth += 1;
            current = self
                .mailboxes
                .get(&id)
                .and_then(|mailbox| mailbox.parent_id.clone());
        }

        depth
    }
}

// Methods altering the cache
impl Cache {
    #[instrument(skip(self))]
    pub fn flush(&mut self) {
        debug!("Flushing cache.");
        self.mailboxes.clear();
        self.children_mapping.clear();
        self.children_query_state.clear();
    }

    #[instrument(skip(self))]
    pub fn add(&mut self, mailbox: MailboxData) {
        let id = mailbox.id.clone();
        self.mailboxes.insert(id.clone(), Arc::new(mailbox.clone()));
    }

    pub fn add_children(&mut self, parent: ParentMailboxId, additional_children: &[MailboxData]) {
        let additional_children_ids: Vec<MailboxId> = additional_children
            .iter()
            .map(|mailbox| mailbox.id.clone())
            .collect();

        for child in additional_children {
            self.add(child.clone());
        }

        self.children_mapping
            .entry(parent)
            .and_modify(|children| {
                for new_child in additional_children {
                    let insert_pos = children.partition_point(|entry| {
                        let child = self.mailboxes.get(entry).unwrap();

                        child.sort_order < new_child.sort_order
                    });

                    children.insert(insert_pos, new_child.id.clone());
                }
            })
            .or_insert(additional_children_ids);
    }

    pub fn remove(&mut self, id: MailboxId) -> Option<Arc<MailboxData>> {
        let mailbox = self.mailboxes.remove(&id)?;
        if let Some(siblings) = self.children_mapping.get_mut(&mailbox.parent_id) {
            siblings.retain(|id| *id != mailbox.id);
        }
        self.children_mapping.remove(&Some(mailbox.id.clone()));
        Some(mailbox)
    }

    pub fn update(&mut self, new: MailboxUpdate) {
        if let (Some(new_name), Some(mailbox)) = (new.name, self.mailboxes.get_mut(&new.id)) {
            Arc::make_mut(mailbox).name = new_name;
        }

        if let (Some(new_role), Some(mailbox)) = (new.role, self.mailboxes.get_mut(&new.id)) {
            Arc::make_mut(mailbox).role = new_role;
        }

        if let (Some(new_sort_order), Some(mailbox)) =
            (new.sort_order, self.mailboxes.get_mut(&new.id))
        {
            Arc::make_mut(mailbox).sort_order = new_sort_order;

            if let Some(siblings) = self.children_mapping.get_mut(&mailbox.parent_id) {
                let old_pos = siblings
                    .iter()
                    .position(|child| child == &mailbox.id)
                    .unwrap();
                let child = siblings.remove(old_pos);

                let new_pos = siblings.partition_point(|sibling| {
                    self.mailboxes.get(sibling).unwrap().sort_order < new_sort_order
                });
                siblings.insert(new_pos, child);
            }
        }

        if let Some(new_parent) = new.parent_id {
            if let Some(mailbox) = self.mailboxes.get(&new.id) {
                // remove from old siblings
                if let Some(children) = self.children_mapping.get_mut(&mailbox.parent_id) {
                    children.retain(|child| child != &mailbox.id);
                }

                // add to new siblings
                self.children_mapping
                    .entry(mailbox.parent_id.clone())
                    .and_modify(|siblings| {
                        let idx = siblings.partition_point(|child| {
                            let other = self.mailboxes.get(child).unwrap();
                            other.sort_order < mailbox.sort_order
                        });

                        siblings.insert(idx, mailbox.id.clone());
                    })
                    .or_insert(vec![mailbox.id.clone()]);
            }

            // finally, update the parent
            if let Some(mailbox) = self.mailboxes.get_mut(&new.id) {
                Arc::make_mut(mailbox).parent_id = new_parent;
            }
        }
    }
}
