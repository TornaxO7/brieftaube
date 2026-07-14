use crate::{backend::mailboxes::MailboxData, ui::MailboxId};
use std::{cmp::Ordering, collections::HashMap};

type MailboxOwner = Option<MailboxId>;

pub struct Layers {
    layers: HashMap<Option<MailboxId>, Layer>,
    selected_layer: Vec<Option<MailboxId>>,
}

impl Layers {
    pub fn new(mailboxes: Vec<MailboxData>) -> Self {
        let mut layers: HashMap<MailboxOwner, Layer> = HashMap::with_capacity(mailboxes.len());

        // create for each mailbox a layer
        {
            // root layer
            layers.insert(None, Layer::new(None));
            for mailbox in mailboxes.clone() {
                let id = mailbox.id.clone();
                layers.insert(Some(id.clone()), Layer::new(Some(id.clone())));
            }
        }

        // add children
        {
            for mailbox in mailboxes {
                let parent_id = &mailbox.parent_id;
                let layer = layers.get_mut(parent_id).expect("Parent layer exists.");
                layer.mailboxes.push(mailbox);
            }
        }

        for layer in layers.values_mut() {
            layer.sort_mailboxes();
        }

        Self {
            layers,
            selected_layer: vec![None],
        }
    }

    pub fn select_next_mailbox(&mut self) {
        self.get_current_layer_mut().list_state.next();
    }

    pub fn select_previous_mailbox(&mut self) {
        self.get_current_layer_mut().list_state.previous();
    }

    pub fn set_sort_order(&mut self, new_order: u32) -> Option<MailboxId> {
        let layer = self.get_current_layer_mut();
        let idx = {
            if !layer.is_root_layer() && layer.selected_parent() {
                return None;
            }

            if layer.is_root_layer() {
                layer.list_state.selected.unwrap()
            } else {
                layer.list_state.selected.unwrap() - 1
            }
        };

        let mailbox_id = {
            let mailbox = layer.mailboxes.get_mut(idx).unwrap();
            mailbox.sort_order = new_order;
            mailbox.id.clone()
        };
        layer.sort_mailboxes();
        Some(mailbox_id)
    }

    pub fn get_current_selected_entry(&self) -> MailboxId {
        let layer = self.get_current_layer();

        let idx = layer.list_state.selected.unwrap();
        if layer.is_root_layer() {
            layer.mailboxes[idx].id.clone()
        } else if layer.selected_parent() {
            layer.mailbox_owner.clone().unwrap()
        } else {
            layer.mailboxes[idx - 1].id.clone()
        }
    }

    pub fn get_current_layer(&self) -> &Layer {
        self.layers
            .get(self.selected_layer.last().unwrap())
            .unwrap()
    }

    pub fn get_current_layer_mut(&mut self) -> &mut Layer {
        self.layers
            .get_mut(self.selected_layer.last().unwrap())
            .unwrap()
    }
}

// For rendering
impl Layers {
    pub fn get_parent_layer_mut(&mut self) -> Option<&mut Layer> {
        let selected_layer_len = self.selected_layer.len();
        let has_parent = self.selected_layer.len() > 1;
        if has_parent {
            let id = &self.selected_layer[selected_layer_len - 2];
            Some(self.layers.get_mut(id).unwrap())
        } else {
            None
        }
    }

    pub fn get_children_layer_mut(&mut self) -> &mut Layer {
        let layer = self.get_current_layer();

        let selected_idx = layer.list_state.selected.unwrap();
        let selected_mailbox = &layer.mailboxes[selected_idx];
        self.layers
            .get_mut(&Some(selected_mailbox.id.clone()))
            .unwrap()
    }
}

#[derive(Clone, Default)]
pub struct Layer {
    pub mailbox_owner: Option<MailboxId>,
    pub mailboxes: Vec<MailboxData>,
    pub list_state: tui_widget_list::ListState,
}

impl Layer {
    pub fn new(mailbox_owner: Option<MailboxId>) -> Self {
        Self {
            mailbox_owner,
            mailboxes: vec![],
            list_state: {
                let mut state = tui_widget_list::ListState::default();
                state.select(Some(0));
                state
            },
        }
    }

    pub fn is_root_layer(&self) -> bool {
        self.mailbox_owner.is_none()
    }

    pub fn selected_parent(&self) -> bool {
        !self.is_root_layer() && self.list_state.selected.map(|idx| idx == 0).unwrap()
    }

    fn sort_mailboxes(&mut self) {
        self.mailboxes.sort_by(|a, b| {
            let ordering = a.sort_order.cmp(&b.sort_order);

            if ordering == Ordering::Equal {
                a.name.cmp(&b.name)
            } else {
                ordering
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};

    fn create_mailbox(id: u32, sort_order: u32, parent_id: Option<String>) -> MailboxData {
        MailboxData {
            id: format!("{}", id),
            sort_order,
            parent_id,
            ..Default::default()
        }
    }

    fn check_layer(layer: &Layer, expected_layer: &Layer) {
        assert_eq!(layer.mailbox_owner, expected_layer.mailbox_owner);

        assert_eq!(layer.mailboxes.len(), expected_layer.mailboxes.len());
        for (mailbox, expected_mailbox) in
            layer.mailboxes.iter().zip(expected_layer.mailboxes.iter())
        {
            assert_eq!(mailbox, expected_mailbox);
        }

        assert!(
            layer
                .mailboxes
                .iter()
                .is_sorted_by(|a, b| a.sort_order <= b.sort_order)
        );
    }

    #[test]
    fn nested() {
        let root_mailboxes: Vec<MailboxData> = (0..2)
            .into_iter()
            .map(|num| create_mailbox(num, num, None))
            .collect();
        let amount_root_mailboxes = root_mailboxes.len();

        let children_of_mailbox_0: Vec<MailboxData> =
            vec![create_mailbox(2, 2, Some("0".to_string()))];
        let amount_children_mailboxes_of_mailbox_0 = children_of_mailbox_0.len();

        let mailboxes = {
            let mut rng = StdRng::seed_from_u64(0);
            let mut mailboxes =
                vec![root_mailboxes.clone(), children_of_mailbox_0.clone()].concat();
            mailboxes.shuffle(&mut rng);
            mailboxes
        };

        let layers = Layers::new(mailboxes);

        assert_eq!(layers.layers.len(), {
            let root_layer = 1;
            root_layer + amount_root_mailboxes + amount_children_mailboxes_of_mailbox_0
        },);

        check_layer(
            layers.layers.get(&None).unwrap(),
            &Layer {
                mailbox_owner: None,
                mailboxes: root_mailboxes.clone(),
                ..Default::default()
            },
        );

        check_layer(
            layers.layers.get(&Some("0".to_string())).unwrap(),
            &Layer {
                mailbox_owner: Some("0".to_string()),
                mailboxes: children_of_mailbox_0.clone(),
                ..Default::default()
            },
        );

        check_layer(
            layers.layers.get(&Some("1".to_string())).unwrap(),
            &Layer {
                mailbox_owner: Some("1".to_string()),
                mailboxes: vec![],
                ..Default::default()
            },
        );

        check_layer(
            layers.layers.get(&Some("2".to_string())).unwrap(),
            &Layer {
                mailbox_owner: Some("2".to_string()),
                mailboxes: vec![],
                ..Default::default()
            },
        );
    }
}
