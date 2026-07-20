use super::types::{Entry, MailboxData, MailboxId};
use crate::backend::mailbox::types::MailboxUpdate;
use jmap_client::core::response::MailboxGetResponse;
use std::collections::HashMap;
use tracing::warn;

pub struct Cache {
    mailboxes: HashMap<MailboxId, MailboxData>,
    children_mapping: HashMap<Option<MailboxId>, Vec<Entry>>,
    get_state: String,
}

impl Cache {
    pub fn new(mut response: MailboxGetResponse) -> Self {
        let raw = response.take_list();
        let get_state = response.take_state();

        let mailboxes: HashMap<MailboxId, MailboxData> = raw
            .iter()
            .map(|mailbox| {
                (
                    mailbox.id().unwrap().to_string(),
                    MailboxData::from(mailbox),
                )
            })
            .collect();

        let children_mapping = {
            let top_mailbox = (None, vec![]);
            let mut mapping: HashMap<Option<MailboxId>, Vec<Entry>> = HashMap::from([top_mailbox]);

            // every mailbox gets its own entries
            for mailbox in mailboxes.values() {
                mapping
                    .entry(mailbox.parent_id.clone())
                    .or_insert(vec![Entry::This]);
                mapping.insert(Some(mailbox.id.clone()), vec![Entry::This]);
            }

            for mailbox in mailboxes.values() {
                let children = mapping
                    .get_mut(&mailbox.parent_id)
                    .expect("Children list exists due to the previous for-loop.");

                let idx = children.partition_point(|entry| match entry {
                    Entry::This => true,
                    Entry::Child(id) => mailboxes.get(id).unwrap().sort_order < mailbox.sort_order,
                });

                children.insert(idx, Entry::Child(mailbox.id.clone()));
            }

            mapping
        };

        Self {
            mailboxes,
            children_mapping,
            get_state,
        }
    }

    pub fn get_current_state(&self) -> String {
        self.get_state.clone()
    }

    pub fn set_state(&mut self, new_state: String) {
        self.get_state = new_state;
    }

    pub fn get_mailbox(&self, id: &MailboxId) -> Option<&MailboxData> {
        self.mailboxes.get(id)
    }

    pub fn get_children(&self, parent_id: &Option<MailboxId>) -> &[Entry] {
        self.children_mapping.get(parent_id).unwrap()
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

    /// Checks if a mailbox with `name` exists among the children of `parent_id`.
    pub fn contains_mailbox_name(&self, parent_id: &Option<MailboxId>, name: &str) -> bool {
        self.children_mapping
            .get(parent_id)
            .is_some_and(|children| {
                children.iter().any(|entry| match entry {
                    Entry::This => false,
                    Entry::Child(id) => self
                        .mailboxes
                        .get(id)
                        .is_some_and(|mailbox| mailbox.name == name),
                })
            })
    }
}

// Actions
impl Cache {
    pub fn remove_mailbox(&mut self, id: &MailboxId) {
        if let Some(mailbox) = self.mailboxes.remove(id) {
            if let Some(siblings) = self.children_mapping.get_mut(&mailbox.parent_id) {
                siblings.retain(|entry| !matches!(entry, Entry::Child(id) if *id == mailbox.id));
            }

            self.children_mapping.remove(&Some(mailbox.id));
        }
    }

    pub fn update_mailbox(&mut self, new: MailboxUpdate) {
        if let Some(new_name) = new.name {
            let old = self.mailboxes.get_mut(&new.id).unwrap();
            old.name = new_name;
        }

        if let Some(new_role) = new.role {
            let old = self.mailboxes.get_mut(&new.id).unwrap();
            old.role = new_role;
        }

        if let Some(new_sort_order) = new.sort_order {
            let (parent_id, id) = {
                let old = self.mailboxes.get_mut(&new.id).unwrap();
                old.sort_order = new_sort_order;
                (old.parent_id.clone(), old.id.clone())
            };

            let children = self.children_mapping.get_mut(&parent_id).unwrap();

            let old_pos = children
                .iter()
                .position(|entry| matches!(entry, Entry::Child(other) if *other == id))
                .unwrap();

            let child = children.remove(old_pos);

            let new_pos = children.partition_point(|entry| match entry {
                Entry::This => true,
                Entry::Child(other) => {
                    self.mailboxes.get(other).unwrap().sort_order < new_sort_order
                }
            });

            children.insert(new_pos, child);
        }

        if let Some(new_parent) = new.parent_id {
            // remove from old siblings
            {
                let prev_parent = self.mailboxes.get(&new.id).unwrap().parent_id.clone();
                let prev_siblings = self.children_mapping.get_mut(&prev_parent).unwrap();
                prev_siblings.retain(|entry| matches!(entry, Entry::Child(id) if id != &new.id));
            }

            // add to new siblings
            {
                let new_siblings = self.children_mapping.get_mut(&new_parent).unwrap();
                let new_pos = new_siblings.partition_point(|entry| match entry {
                    Entry::This => false,
                    Entry::Child(other_id) => {
                        let other = self.mailboxes.get(other_id).unwrap();
                        let old = self.mailboxes.get(&new.id).unwrap();

                        other.sort_order < old.sort_order
                    }
                });

                new_siblings.insert(new_pos, Entry::Child(new.id.clone()));
            }

            // finally, update the parent
            let old = self.mailboxes.get_mut(&new.id).unwrap();
            old.parent_id = new_parent;
        }
    }

    pub fn add_new_mailbox(&mut self, mailbox: MailboxData) {
        self.children_mapping
            .insert(Some(mailbox.id.clone()), vec![Entry::This]);

        self.children_mapping
            .get_mut(&mailbox.parent_id)
            .map(|children| {
                let idx = children.partition_point(|entry| match entry {
                    Entry::This => true,
                    Entry::Child(other) => {
                        self.mailboxes.get(other).unwrap().sort_order < mailbox.sort_order
                    }
                });

                children.insert(idx, Entry::Child(mailbox.id.clone()));
            });

        self.mailboxes.insert(mailbox.id.clone(), mailbox);
    }
}
