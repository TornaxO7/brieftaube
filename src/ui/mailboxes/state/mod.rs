mod layer;

use super::Action;
use crate::{
    backend::Account,
    ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState,
        utils::{self, keybindmanager::KeybindManager},
    },
};
pub use layer::Layers;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, trace};

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {
    SortOrder,
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
                ("<CR>", Action::OpenSelectedMailbox),
                ("l", Action::OpenSelectedMailbox),
            ])),
        }
    }
}

impl ScreenState<Action, PaletteValue, InputType> for State {
    #[tracing::instrument(level = "debug", skip_all)]
    fn update(&mut self) {
        if let Some((mailboxes, new_state)) = self.account.get_mailboxes(&self.data_state) {
            trace!("Updating");

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
            Action::OpenSelectedMailbox => todo!(),
            Action::SetSortOrder => {
                self.overlay = Some(ScreenOverlay::Input(utils::input::State::new(
                    "Set sort order (>= 0):",
                    InputType::SortOrder,
                )));
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
                            Ok(new_order) => {
                                let id = layers.set_sort_order(new_order);
                                self.account.update_mailbox_sort_order(id, new_order);
                            }
                            Err(err) => {
                                error!("'{}' isn't a 32-bit unsigned integer: {}", value, err)
                            }
                        }
                    }
                }
            },
            ScreenOverlayResult::Cancel => {}
        }
    }
}
