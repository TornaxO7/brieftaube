mod layer;

use super::Action;
use crate::{
    backend::{Account, mailboxes::MailboxData},
    ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState,
        utils::{self, keybindmanager::KeybindManager},
    },
};
pub use layer::{Layer, Layers};
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, error, trace};

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {
    SortOrder,
    NewMailboxName,
}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    account: Arc<Account>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    pub layers: Option<Layers>,

    data_state: String,
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            app_actions: vec![],
            account,
            layers: None,

            data_state: String::new(),
            overlay: None,

            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("j", Action::SelectNextMailbox),
                ("k", Action::SelectPreviousMailbox),
                // ("n", super::Action::OpenComposer),
                (":", Action::OpenCommandPalette),
                ("<CR>", Action::ActivateSelectedEntry),
                ("l", Action::ActivateSelectedEntry),
                ("h", Action::GoBack),
            ])),
        }
    }
}

impl ScreenState<Action, PaletteValue, InputType> for State {
    #[tracing::instrument(level = "debug", skip_all)]
    fn update(&mut self) {
        if let Some((mailboxes, new_state)) = self.account.get_mailboxes(&self.data_state) {
            trace!("Updating");
            debug!("{:#?}", mailboxes);

            self.layers = Some(Layers::new(mailboxes));
            self.data_state = new_state;
        }
    }

    fn apply_action(&mut self, action: Action) {
        trace!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(utils::palette::State::new(
                    super::action::palette_options(),
                )))
            }

            Action::SelectNextMailbox => {
                if let Some(layers) = self.layers.as_mut() {
                    layers.select_next_mailbox();
                }
            }
            Action::SelectPreviousMailbox => {
                if let Some(layers) = self.layers.as_mut() {
                    layers.select_previous_mailbox();
                }
            }
            Action::ActivateSelectedEntry => {
                if let Some(layers) = self.layers.as_mut() {
                    if let Some(id) = layers.open_selected_entry() {
                        self.app_actions.push(crate::Action::OpenRootMails(id));
                    }
                }
            }
            Action::GoBack => {
                if let Some(layers) = self.layers.as_mut() {
                    layers.go_up_one_level();
                }
            }
            Action::SetSortOrder => {
                if let Some(layers) = self.layers.as_ref() {
                    let layer = layers.get_current_layer();

                    if layer.selected_parent() {
                        error!(
                            "You can't set the sort order of the parent id. Move up one mailbox first."
                        );
                    } else {
                        self.overlay = Some(ScreenOverlay::Input(utils::input::State::new(
                            "Set sort order (>= 0):",
                            InputType::SortOrder,
                        )));
                    }
                }
            }
            Action::CreateMailbox => {
                self.overlay = Some(ScreenOverlay::Input(utils::input::State::new(
                    "Create:",
                    InputType::NewMailboxName,
                )));
            }
            Action::DestroyMailbox => {
                if let Some(layers) = self.layers.as_ref() {
                    let layer = layers.get_current_layer();

                    match layer.get_selected_mailbox() {
                        Some(selected_mailbox) => {
                            self.account.destroy_mailbox(selected_mailbox.id.clone());
                        }
                        None => {
                            error!(
                                "Can't destroy mailbox: You can't select the parent mailbox to destroy it."
                            );
                        }
                    }
                }
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut ScreenOverlay<PaletteValue, InputType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: ScreenOverlayResult<PaletteValue, InputType>) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Palette(value) => match value {
                PaletteValue::Action(action) => self.apply_action(action),
            },
            ScreenOverlayResult::Input { value, typ } => match typ {
                InputType::SortOrder => {
                    if let Some(layers) = self.layers.as_mut() {
                        match value.parse::<u32>() {
                            Ok(new_order) => match layers.set_sort_order(new_order) {
                                Some(id) => {
                                    self.account.update_mailbox_sort_order(id, new_order);
                                }
                                None => {
                                    unreachable!(
                                        "A check should have happened before that the current selected mailbox isn't a parent directory..."
                                    );
                                }
                            },
                            Err(err) => {
                                error!(
                                    "Can't set sor order: {}' isn't a 32-bit unsigned integer: {}",
                                    value, err
                                )
                            }
                        }
                    }
                }
                InputType::NewMailboxName => {
                    if let Some(layers) = self.layers.as_mut() {
                        // validate
                        {
                            let caps = self.account.mail_capability();

                            let msg = {
                                if layers.depth() > caps.max_mailbox_depth() {
                                    Some(format!(
                                        "Max mailbox depth reached for the mail server :( You can't create another sub-mailbox in the current mailbox. The maximum depth is {} for the server.",
                                        caps.max_mailbox_depth()
                                    ))
                                } else if value.len() > caps.max_size_mailbox_name() {
                                    Some(format!(
                                        "The mailbox name is too long. It can be at most {} characters long.",
                                        caps.max_size_mailbox_name()
                                    ))
                                } else if layers
                                    .get_current_layer()
                                    .contains_mailbox_name(value.as_str())
                                {
                                    Some(format!(
                                        "There's already a mailbox in the current mailbox with the name '{}'.",
                                        &value
                                    ))
                                } else {
                                    None
                                }
                            };

                            if let Some(msg) = msg {
                                error!("Can't create mailbox: {}", msg);
                                return;
                            }
                        }

                        let layer = layers.get_current_layer();
                        let new_mailbox_data = MailboxData {
                            name: value,
                            parent_id: layer.mailbox_owner.clone(),
                            sort_order: layer
                                .mailboxes
                                .iter()
                                .map(|mailbox| mailbox.sort_order)
                                .max()
                                .map(|biggest_sort_order| biggest_sort_order + 1)
                                .unwrap_or(0),
                            ..Default::default()
                        };
                        self.account.create_mailbox(new_mailbox_data);
                    }
                }
            },
            ScreenOverlayResult::Cancel => {}
        }
    }
}
