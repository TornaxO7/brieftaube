use super::Action;
use crate::{
    mailboxes::backend::Backend,
    utils::{
        MailboxId,
        ui::{
            ScreenOverlay, ScreenOverlayResult, ScreenState, input, keybindmanager::KeybindManager,
            palette,
        },
    },
};
use std::{collections::HashMap, rc::Rc};
use tracing::{error, trace};

#[derive(Debug, Clone)]
pub enum PaletteValue {
    Action(Action),
}

#[derive(Debug, Clone)]
pub enum InputType {
    SortOrder(MailboxId),
    NewMailboxName { parent: Option<MailboxId> },
}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    overlay: Option<ScreenOverlay<PaletteValue, InputType>>,

    pub backend: Rc<Backend>,
}

impl State {
    pub fn new(backend: Rc<Backend>) -> Self {
        backend.init();

        Self {
            app_actions: vec![],
            backend,

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
    fn apply_action(&mut self, action: Action) {
        trace!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )))
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::SelectNextMailbox => self.backend.select_next_mailbox(),
            Action::SelectPreviousMailbox => self.backend.select_previous_mailbox(),
            Action::ActivateSelectedEntry => {
                if let Some(id) = self.backend.activate_selected_entry() {
                    self.app_actions.push(crate::Action::OpenRootMails(id));
                }
            }
            Action::GoBack => self.backend.go_back(),
            Action::SetSortOrder => {
                if let Some(can_set_sort_order) = self.backend.can_set_sort_order() {
                    if can_set_sort_order {
                        let Some(id) = self.backend.get_selected_mailbox() else {
                            return;
                        };

                        self.overlay = Some(ScreenOverlay::Input(input::State::new(
                            "Set sort order (>= 0):",
                            InputType::SortOrder(id),
                        )));
                    } else {
                        error!(
                            "You can't set the sort order of the parent id. Move up one mailbox first."
                        );
                    }
                }
            }
            Action::MoveMailboxUp => {
                todo!()
            }
            Action::MoveMailboxDown => {
                todo!()
            }
            Action::CreateMailbox => {
                let parent = self.backend.get_parent_mailbox();

                self.overlay = Some(ScreenOverlay::Input(input::State::new(
                    "Create:",
                    InputType::NewMailboxName { parent },
                )));
            }
            Action::DestroySelectedMailbox => self.backend.destroy_selected_mailbox(),
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
                InputType::SortOrder(id) => match value.parse::<u32>() {
                    Ok(new_order) => self.backend.set_new_order(id, new_order),
                    Err(err) => {
                        error!(
                            "Can't set sort order: {}' isn't a 32-bit unsigned integer: {}",
                            value, err
                        )
                    }
                },
                InputType::NewMailboxName { parent } => self.backend.create_mailbox(parent, value),
            },
            ScreenOverlayResult::Cancel => {}
        }
    }
}
