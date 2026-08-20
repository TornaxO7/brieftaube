use super::types::{MailboxData, MailboxId};
use crate::backend::{
    GetState, QueryState,
    mailbox::types::{MailboxUpdate, ParentMailboxId},
    mails::types::MailId,
    types::Loadable,
};
use jmap_client::core::query::QueryResponse;
use std::collections::HashMap;

/// Convention: Any available Id means that its data exists in `mailboxes`
pub struct Store {
    mailboxes: HashMap<MailboxId, MailboxData>,
    // Each `MailboxId` has an entry in the `mailboxes` attribute.
    // `Vec` is **unsorted**
    children_mapping: HashMap<ParentMailboxId, Loadable<Vec<MailboxId>>>,
    root_mails: HashMap<MailboxId, Loadable<RootMails>>,

    /// stores the query state of the child-mailboxes of the given parent-mailbox
    children_query_state: HashMap<ParentMailboxId, QueryState>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            children_mapping: HashMap::new(),
            children_query_state: HashMap::new(),
            root_mails: HashMap::new(),
        }
    }

    pub fn get_data(&self, id: &MailboxId) -> &MailboxData {
        self.mailboxes.get(id).unwrap()
    }

    pub fn get_data_mut(&mut self, id: &MailboxId) -> &mut MailboxData {
        self.mailboxes.get_mut(id).unwrap()
    }
}

// root mails
impl Store {
    pub fn get_root_mails(&mut self, parent: &MailboxId) -> &Loadable<RootMails> {
        self.get_root_mails_mut(parent)
    }

    pub fn get_root_mails_mut(&mut self, parent: &MailboxId) -> &mut Loadable<RootMails> {
        self.root_mails
            .entry(parent.clone())
            .or_insert(Loadable::NotRequested)
    }

    pub fn set_root_mails(&mut self, parent: MailboxId, root_mails: RootMails) {
        self.root_mails
            .insert(parent, Loadable::Loaded(root_mails));
    }
}

// children ids
impl Store {
    pub fn set_children_query_state(&mut self, parent: ParentMailboxId, new_state: GetState) {
        self.children_query_state.insert(parent, new_state);
    }

    pub fn get_children_ids(&mut self, parent_id: &ParentMailboxId) -> &Loadable<Vec<MailboxId>> {
        self.get_children_ids_mut(parent_id)
    }

    pub fn get_children_ids_mut(
        &mut self,
        parent_id: &ParentMailboxId,
    ) -> &mut Loadable<Vec<MailboxId>> {
        self.children_mapping
            .entry(parent_id.clone())
            .or_insert(Loadable::NotRequested)
    }
}

/// validation
impl Store {
    /// Depth of the given parent (root = 0, its children = 1, ...).
    ///
    /// Panics if any of the parent mailboxes aren't loaded yet.
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

    /// Panics if children haven't been loaded yet.
    pub fn contains_mailbox_name(&self, parent: &ParentMailboxId, name: &str) -> bool {
        let Some(children) = self.children_mapping.get(parent) else {
            return false;
        };

        for child_id in children.loaded().unwrap() {
            let child = self.get_data(child_id);

            if child.name == name {
                return true;
            }
        }

        false
    }
}

// Methods altering the cache
impl Store {
    pub fn add_children(&mut self, parent: &ParentMailboxId, children: Vec<MailboxData>) {
        let children_ids: Vec<MailboxId> = children.iter().map(|data| data.id.clone()).collect();

        self.children_mapping
            .insert(parent.clone(), Loadable::Loaded(children_ids));

        for child in children {
            self.mailboxes.insert(child.id.clone(), child);
        }
    }

    pub fn remove(&mut self, id: &MailboxId) -> Option<MailboxData> {
        let mailbox = self.mailboxes.remove(id)?;

        let siblings = self
            .children_mapping
            .get_mut(&mailbox.parent_id)
            .expect("`add_children` only allows that we fetch all siblings, so it must exist")
            .loaded_mut()
            .expect("Siblings must be loaded");

        let pos = siblings
            .iter()
            .position(|other| other == id)
            .expect("It's stored that they are siblings...");

        siblings.remove(pos);

        self.children_query_state.remove(&Some(id.clone()));
        self.children_mapping.remove(&Some(mailbox.id.clone()));
        self.root_mails.remove(id);
        Some(mailbox)
    }

    pub fn update(&mut self, new: MailboxUpdate) {
        if let Some(new_name) = new.name {
            let mailbox = self.get_data_mut(&new.id);
            mailbox.name = new_name;
        }

        if let Some(new_role) = new.role {
            let mailbox = self.get_data_mut(&new.id);
            mailbox.role = new_role;
        }

        if let Some(new_sort_order) = new.sort_order {
            let mailbox = self.get_data_mut(&new.id);
            mailbox.sort_order = new_sort_order;
        }

        if let Some(new_parent) = new.parent_id {
            let current_parent = {
                let mailbox = self.get_data(&new.id);
                mailbox.parent_id.clone()
            };

            // remove from old siblings
            {
                let siblings = self
                    .get_children_ids_mut(&current_parent)
                    .loaded_mut()
                    .unwrap();

                let pos = siblings.iter().position(|other| other == &new.id).unwrap();
                siblings.remove(pos);
            }

            // add to new siblings
            {
                let siblings = self.get_children_ids_mut(&new_parent).loaded_mut().unwrap();
                siblings.push(new.id.clone());
            }

            // finally, update the parent
            let mailbox = self.get_data_mut(&new.id);
            mailbox.parent_id = new_parent;
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootMails {
    pub ids: Vec<MailId>,
    pub _state: QueryState,
}

impl RootMails {
    pub fn new(mut response: QueryResponse) -> Self {
        let state = response.take_query_state();
        let ids = response.take_ids().into_iter().map(MailId).collect();

        Self { ids, _state: state }
    }
}
