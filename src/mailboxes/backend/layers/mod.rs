mod layer;

use std::collections::HashMap;

use super::MailboxData;
use crate::utils::MailboxId;
pub use layer::Layer;

type MailboxOwner = Option<MailboxId>;

mod err_msg {
    pub const NON_EMPTY_SELECTED_LAYER: &str = "`selected_layer` can't become empty!";
    pub const MAILBOX_HAS_LAYER: &str = "Every mailbox must have a layer.";
    pub const MAILBOX_IS_IN_LAYER: &str = "Each mailbox must be in a layer.";
}

pub struct Layers {
    // each mailbox has its own layer
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

    pub fn depth(&self) -> usize {
        self.selected_layer.len() - 1
    }

    pub fn set_sort_order(&mut self, id: MailboxId, new_order: u32) {
        let mailbox = self.get_mailbox_mut(&id).unwrap();
        mailbox.sort_order = new_order;

        let layer = self.get_layer_containing_mailbox_mut(&id);
        layer.sort_mailboxes();
    }

    pub fn open_selected_entry(&mut self) -> Option<MailboxId> {
        let layer = self.get_current_layer();

        if layer.selected_parent() {
            return layer.mailbox_owner.clone();
        } else {
            let idx = layer.state.selected().unwrap();
            let selected_mailbox = if layer.is_root_layer() {
                layer.mailboxes[idx].id.clone()
            } else {
                layer.mailboxes[idx - 1].id.clone()
            };

            self.selected_layer.push(Some(selected_mailbox));
            None
        }
    }

    pub fn add_mailbox(&mut self, mailbox: MailboxData) {
        self.layers.insert(
            Some(mailbox.id.clone()),
            Layer::new(Some(mailbox.id.clone())),
        );

        let parent = self.layers.get_mut(&mailbox.parent_id).unwrap();
        parent.mailboxes.push(mailbox);
        parent.sort_mailboxes();
    }

    pub fn go_up_one_level(&mut self) {
        if self.selected_layer.len() > 1 {
            self.selected_layer.pop();
        }
    }

    pub fn get_current_layer(&self) -> &Layer {
        self.layers
            .get(self.selected_layer.last().unwrap())
            .expect(err_msg::NON_EMPTY_SELECTED_LAYER)
    }

    pub fn get_current_layer_mut(&mut self) -> &mut Layer {
        self.layers
            .get_mut(self.selected_layer.last().unwrap())
            .expect(err_msg::NON_EMPTY_SELECTED_LAYER)
    }

    pub fn get_layer(&self, id: &Option<MailboxId>) -> &Layer {
        self.layers.get(id).expect(err_msg::MAILBOX_HAS_LAYER)
    }

    pub fn get_layer_mut(&mut self, id: &Option<MailboxId>) -> &mut Layer {
        self.layers.get_mut(id).expect(err_msg::MAILBOX_HAS_LAYER)
    }

    pub fn get_mailbox(&self, id: &MailboxId) -> Option<&MailboxData> {
        self.layers.values().find_map(|layer| layer.get_mailbox(id))
    }

    pub fn get_mailbox_mut(&mut self, id: &MailboxId) -> Option<&mut MailboxData> {
        self.layers
            .values_mut()
            .find_map(|layer| layer.get_mailbox_mut(id))
    }

    pub fn get_layer_containing_mailbox(&self, id: &MailboxId) -> &Layer {
        self.layers
            .values()
            .find(|layer| layer.get_mailbox(id).is_some())
            .expect(err_msg::MAILBOX_IS_IN_LAYER)
    }

    pub fn get_layer_containing_mailbox_mut(&mut self, id: &MailboxId) -> &mut Layer {
        self.layers
            .values_mut()
            .find(|layer| layer.get_mailbox(id).is_some())
            .expect(err_msg::MAILBOX_IS_IN_LAYER)
    }

    pub fn remove_mailbox(&mut self, id: MailboxId) {
        self.layers.remove(&Some(id.clone()));

        let parent_layer = self.get_layer_containing_mailbox_mut(&id);
        parent_layer.mailboxes.retain(|mailbox| mailbox.id != id);
    }

    pub fn remove_layer(&mut self, id: &Option<MailboxId>) -> Layer {
        self.layers.remove(id).expect(err_msg::MAILBOX_HAS_LAYER)
    }

    pub fn insert_layer(&mut self, owner: Option<MailboxId>, layer: Layer) {
        self.layers.insert(owner, layer);
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

    pub fn get_children_layer_mut(&mut self) -> Option<&mut Layer> {
        let layer = self.get_current_layer();

        let id = layer
            .get_selected_mailbox()
            .map(|mailbox| mailbox.id.clone());

        self.layers.get_mut(&id)
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
