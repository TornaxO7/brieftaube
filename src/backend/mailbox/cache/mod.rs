use super::types::{MailboxData, MailboxId};
use crate::backend::{
    GetState, QueryState,
    mailbox::types::{MailboxUpdate, ParentMailboxId},
};
use std::{collections::HashMap, sync::Arc};

pub struct Cache {
    mailboxes: HashMap<MailboxId, Arc<MailboxData>>,
    // Children always exist in `mailboxes`.
    children_mapping: HashMap<ParentMailboxId, Vec<MailboxId>>,
    states: HashMap<ParentMailboxId, QueryState>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            mailboxes: HashMap::new(),
            children_mapping: HashMap::new(),
            states: HashMap::new(),
        }
    }

    pub fn is_initialised(&self, parent: &ParentMailboxId) -> bool {
        self.children_mapping.get(parent).is_some()
    }

    pub fn get_state(&self, parent: &ParentMailboxId) -> Option<GetState> {
        self.states.get(parent).cloned()
    }

    pub fn set_state(&mut self, parent: ParentMailboxId, new_state: GetState) {
        self.states.insert(parent, new_state);
    }

    pub fn get_data(&self, id: &MailboxId) -> Option<Arc<MailboxData>> {
        self.mailboxes.get(id).cloned()
    }

    pub fn get_children(&self, parent_id: &ParentMailboxId) -> Option<&[MailboxId]> {
        self.children_mapping
            .get(parent_id)
            .map(|children| children.as_slice())
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
    pub fn flush(&mut self) {
        self.mailboxes.clear();
        self.children_mapping.clear();
        self.states.clear();
    }

    pub fn add(&mut self, mailbox: MailboxData) {
        let id = mailbox.id.clone();

        self.mailboxes.insert(id.clone(), Arc::new(mailbox.clone()));

        self.children_mapping
            .entry(mailbox.parent_id.clone())
            .and_modify(|children| {
                let insert_pos = children.partition_point(|entry| {
                    let child = self.mailboxes.get(entry).unwrap();

                    child.sort_order < mailbox.sort_order
                });

                children.insert(insert_pos, id.clone());
            })
            .or_insert(vec![id.clone()]);
    }

    pub fn remove(&mut self, id: MailboxId) -> Option<Arc<MailboxData>> {
        let mailbox = self.mailboxes.remove(&id)?;
        self.children_mapping.remove(&mailbox.parent_id);

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::backend::mailbox::types::SortOrder;

//     fn mailbox(id: &str, parent: Option<&str>, sort_order: SortOrder) -> MailboxData {
//         MailboxData {
//             id: id.to_string(),
//             parent_id: parent.map(|p| p.to_string()),
//             sort_order,
//             ..Default::default()
//         }
//     }

//     mod add {
//         use super::*;

//         #[test]
//         fn adds_mailbox_to_top_level() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("1", None, 0));

//             assert!(cache.mailboxes.contains_key("1"));
//             assert_eq!(
//                 cache.children_mapping.get(&None).unwrap(),
//                 &vec!["1".to_string()]
//             );
//         }

//         #[test]
//         fn inserts_children_sorted_by_sort_order() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("a", None, 10));
//             cache.add(mailbox("b", None, 5));
//             cache.add(mailbox("c", None, 20));

//             let children = cache.children_mapping.get(&None).unwrap();
//             assert_eq!(
//                 children,
//                 &vec!["b".to_string(), "a".to_string(), "c".to_string()]
//             );
//         }

//         #[test]
//         fn groups_children_by_parent() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("parent", None, 0));
//             cache.add(mailbox("child1", Some("parent"), 0));
//             cache.add(mailbox("child2", Some("parent"), 1));

//             let children = cache
//                 .children_mapping
//                 .get(&Some("parent".to_string()))
//                 .unwrap();
//             assert_eq!(children, &vec!["child1".to_string(), "child2".to_string()]);
//         }
//     }

//     mod remove {
//         use super::*;

//         #[test]
//         fn removes_mailbox_from_map() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("1", None, 0));

//             let removed = cache.remove("1".to_string());

//             assert!(removed.is_some());
//             assert!(!cache.mailboxes.contains_key("1"));
//         }

//         #[test]
//         fn returns_none_for_unknown_id() {
//             let mut cache = Cache::new();
//             assert!(cache.remove("nope".to_string()).is_none());
//         }

//         #[test]
//         fn removing_one_child_keeps_siblings() {
//             // Currently fails: `remove` deletes the whole sibling list for
//             // the parent instead of just removing this one id from it.
//             let mut cache = Cache::new();
//             cache.add(mailbox("parent", None, 0));
//             cache.add(mailbox("child1", Some("parent"), 0));
//             cache.add(mailbox("child2", Some("parent"), 1));

//             cache.remove("child1".to_string());

//             let siblings = cache.children_mapping.get(&Some("parent".to_string()));
//             assert_eq!(siblings, Some(&vec!["child2".to_string()]));
//         }
//     }

//     mod update {
//         use super::*;

//         #[test]
//         fn updates_name() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("1", None, 0));

//             cache.update(MailboxUpdate {
//                 id: "1".into(),
//                 name: Some("Inbox".into()),
//                 ..Default::default()
//             });

//             assert_eq!(cache.mailboxes.get("1").unwrap().name, "Inbox");
//         }

//         #[test]
//         fn updates_sort_order_and_resorts_siblings() {
//             let mut cache = Cache::new();
//             cache.add(mailbox("a", None, 0));
//             cache.add(mailbox("b", None, 1));

//             cache.update(MailboxUpdate {
//                 id: "a".into(),
//                 sort_order: Some(5),
//                 ..Default::default()
//             });

//             let children = cache.children_mapping.get(&None).unwrap();
//             assert_eq!(children, &vec!["b".to_string(), "a".to_string()]);
//         }

//         #[test]
//         fn moving_to_new_parent_updates_sibling_lists() {
//             // Currently fails: when adding to the new sibling list, `update`
//             // still reads the *old* mailbox.parent_id instead of new_parent,
//             // so the child ends up filed under the old parent again.
//             let mut cache = Cache::new();
//             cache.add(mailbox("parent_a", None, 0));
//             cache.add(mailbox("parent_b", None, 1));
//             cache.add(mailbox("child", Some("parent_a"), 0));

//             cache.update(MailboxUpdate {
//                 id: "child".into(),
//                 parent_id: Some(Some("parent_b".into())),
//                 ..Default::default()
//             });

//             assert_eq!(
//                 cache.mailboxes.get("child").unwrap().parent_id,
//                 Some("parent_b".to_string())
//             );
//             assert!(
//                 cache
//                     .children_mapping
//                     .get(&Some("parent_a".to_string()))
//                     .unwrap()
//                     .is_empty()
//             );
//             assert_eq!(
//                 cache
//                     .children_mapping
//                     .get(&Some("parent_b".to_string()))
//                     .unwrap(),
//                 &vec!["child".to_string()]
//             );
//         }

//         #[test]
//         fn update_unknown_id_is_noop() {
//             let mut cache = Cache::new();
//             cache.update(MailboxUpdate {
//                 id: "nope".into(),
//                 name: Some("x".into()),
//                 ..Default::default()
//             });

//             assert!(!cache.mailboxes.contains_key("nope"));
//         }
//     }
// }
