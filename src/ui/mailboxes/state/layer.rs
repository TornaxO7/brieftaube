use crate::{backend::mailboxes::MailboxData, ui::MailboxId};
use std::collections::HashMap;

type LayerIdx = usize;
type EntryIdx = usize;
type ParentId = MailboxId;

pub struct Layers {
    layers: Vec<Layer>,
    selected_layer: Vec<usize>,
}

impl Layers {
    pub fn new(mut mailboxes: Vec<MailboxData>) -> Self {
        let mut layers: Vec<Layer> = Vec::new();
        let mut parent_mapping: HashMap<ParentId, LayerIdx> = HashMap::new();
        let mut mailbox_mapping: HashMap<MailboxId, (LayerIdx, EntryIdx)> = HashMap::new();

        // root mailboxes should be the only mailboxes in the first layer
        {
            let mut root_layer = Layer::new();
            let root_mailboxes = mailboxes.extract_if(.., |mailbox| mailbox.parent_id.is_none());

            for (idx, root_mailbox) in root_mailboxes.enumerate() {
                let id = root_mailbox.id.clone();

                mailbox_mapping.insert(id, (0, idx));
                root_layer.push_mailbox(root_mailbox);
            }

            layers.push(root_layer);
        }

        for mailbox in mailboxes {
            let mailbox_id = mailbox.id.clone();
            let parent_id = mailbox
                .parent_id
                .as_ref()
                .expect("Root mailboxes are already extracted.");

            match parent_mapping.get(parent_id).cloned() {
                // we already have a layer for mailboxes which have the same parent mailbox
                Some(layer_idx) => {
                    let layer = layers.get_mut(layer_idx).unwrap();

                    mailbox_mapping.insert(mailbox_id, (layer_idx, layer.children.len()));

                    layer.push_mailbox(mailbox);
                }
                // there is no layer yet, for mailboxes with that parent id.
                None => {
                    let layer_idx = layers.len();

                    parent_mapping.insert(parent_id.clone(), layer_idx);
                    mailbox_mapping.insert(mailbox_id, (layer_idx, 0));

                    layers.push(Layer::from(mailbox));
                }
            }
        }

        // update the `children` list for each layer
        for (parent_id, layer_children_idx) in parent_mapping.into_iter() {
            let (parent_layer_idx, parent_entry_idx) =
                mailbox_mapping.get(&parent_id).cloned().unwrap();

            layers[parent_layer_idx].children[parent_entry_idx] = Some(layer_children_idx);
        }

        for layer in layers.iter_mut() {
            layer.sort_mailboxes();
        }

        Self {
            layers,
            selected_layer: vec![0],
        }
    }

    pub fn select_next_mailbox(&mut self) {
        self.get_mut_current_layer().list_state.next();
    }

    pub fn select_previous_mailbox(&mut self) {
        self.get_mut_current_layer().list_state.previous();
    }

    pub fn set_sort_order(&mut self, new_order: u32) -> MailboxId {
        let layer = self.get_mut_current_layer();

        let idx = layer.list_state.selected.unwrap();
        let mailbox_id = {
            let mailbox = layer.mailboxes.get_mut(idx).unwrap();
            mailbox.sort_order = new_order;
            mailbox.id.clone()
        };
        layer.sort_mailboxes();
        mailbox_id
    }

    fn get_current_layer(&self) -> &Layer {
        &self.layers[*self.selected_layer.last().unwrap()]
    }

    fn get_mut_current_layer(&mut self) -> &mut Layer {
        &mut self.layers[*self.selected_layer.last().unwrap()]
    }
}

// For rendering
impl Layers {
    pub fn parent_layer_render_data(
        &mut self,
    ) -> Option<(&[MailboxData], &mut tui_widget_list::ListState)> {
        let selected_layer_len = self.selected_layer.len();
        let has_parent = self.selected_layer.len() > 1;
        if has_parent {
            let idx = self.selected_layer[selected_layer_len - 2];
            let layer = &mut self.layers[idx];
            Some((&layer.mailboxes, &mut layer.list_state))
        } else {
            None
        }
    }

    pub fn current_layer_render_data(
        &mut self,
    ) -> (&[MailboxData], &mut tui_widget_list::ListState) {
        let layer = self.get_mut_current_layer();
        (&layer.mailboxes, &mut layer.list_state)
    }

    pub fn current_selected_children_mailboxes(
        &mut self,
    ) -> Option<(&[MailboxData], &mut tui_widget_list::ListState)> {
        let layer = self.get_current_layer();

        let selected_idx = layer.list_state.selected.unwrap();
        if let Some(child_layer) = layer.children[selected_idx] {
            let child_layer = &mut self.layers[child_layer];

            Some((&child_layer.mailboxes, &mut child_layer.list_state))
        } else {
            None
        }
    }
}

struct Layer {
    mailboxes: Vec<MailboxData>,
    children: Vec<Option<usize>>,

    list_state: tui_widget_list::ListState,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            mailboxes: vec![],
            children: vec![],
            list_state: tui_widget_list::ListState::default(),
        }
    }

    fn push_mailbox(&mut self, mailbox: MailboxData) {
        self.mailboxes.push(mailbox);
        self.children.push(None);

        if self.list_state.selected.is_none() {
            self.list_state.select(Some(0));
        }
    }

    fn sort_mailboxes(&mut self) {
        let mut mailboxes: Vec<(usize, MailboxData)> =
            self.mailboxes.clone().into_iter().enumerate().collect();

        mailboxes.sort_by(|a, b| a.1.sort_order.cmp(&b.1.sort_order));

        let new_children: Vec<Option<usize>> = mailboxes
            .iter()
            .map(|(idx, _)| self.children[*idx].clone())
            .collect();

        let new_mailboxes: Vec<MailboxData> =
            mailboxes.into_iter().map(|(_, mailbox)| mailbox).collect();

        self.mailboxes = new_mailboxes;
        self.children = new_children;
    }
}

impl From<MailboxData> for Layer {
    fn from(mailbox: MailboxData) -> Self {
        Self {
            mailboxes: vec![mailbox],
            children: vec![None],
            list_state: {
                let mut state = tui_widget_list::ListState::default();
                state.select(Some(0));
                state
            },
        }
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

    fn check_layer(
        layer: &Layer,
        expected_parent_id: Option<ParentId>,
        expected_children: Vec<Option<usize>>,
    ) {
        for mailbox in layer.mailboxes.iter() {
            assert_eq!(mailbox.parent_id, expected_parent_id)
        }

        assert_eq!(layer.children, expected_children);

        assert!(
            layer
                .mailboxes
                .iter()
                .is_sorted_by(|a, b| a.sort_order <= b.sort_order)
        );
    }

    #[test]
    fn nested() {
        let mailboxes = {
            let mut rng = StdRng::seed_from_u64(0);

            let root_layer: Vec<MailboxData> = (0..5)
                .into_iter()
                .map(|num| create_mailbox(num, num, None))
                .collect();

            let children_of_mailbox_0: Vec<MailboxData> = (5..8)
                .into_iter()
                .map(|num| create_mailbox(num, num - 5, Some("0".to_string())))
                .collect();

            let children_of_mailbox_3: Vec<MailboxData> = (8..11)
                .into_iter()
                .map(|num| create_mailbox(num, num - 8, Some("3".to_string())))
                .collect();

            let mut mailboxes =
                vec![root_layer, children_of_mailbox_0, children_of_mailbox_3].concat();
            mailboxes.shuffle(&mut rng);
            mailboxes
        };

        let layers = Layers::new(mailboxes);

        assert_eq!(
            layers.layers.len(),
            3,
            "We have three layers: One root layer, one layer under mailbox '0' and another under mailbox '3'."
        );

        check_layer(
            &layers.layers[0],
            None,
            vec![Some(1), None, None, Some(2), None],
        );
        check_layer(&layers.layers[1], Some("0".to_string()), vec![None; 3]);
        check_layer(&layers.layers[2], Some("3".to_string()), vec![None; 3]);
    }
}
