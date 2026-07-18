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
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
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

    is_in_select_mode: bool,
    pub selected: HashSet<MailboxId>,
    pub backend: Rc<Backend>,
}

impl State {
    pub fn new(backend: Rc<Backend>) -> Self {
        backend.init();

        Self {
            app_actions: vec![],
            backend,

            overlay: None,
            selected: HashSet::new(),
            is_in_select_mode: false,

            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                ("j", Action::SelectNextMailbox),
                ("k", Action::SelectPreviousMailbox),
                // ("n", super::Action::OpenComposer),
                (":", Action::OpenCommandPalette),
                ("<CR>", Action::ActivateSelectedEntry),
                (" ", Action::ToggleMailbox),
                ("l", Action::ActivateSelectedEntry),
                ("h", Action::GoBack),
                ("<C-l>", Action::OpenLogs),
                ("gg", Action::SelectTopMailbox),
                ("ge", Action::SelectBottomMailbox),
                ("v", Action::EnterSelectMode),
                ("<ESC>", Action::LeaveSelectMode),
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
            Action::SelectNextMailbox => {
                if self.is_in_select_mode {
                    if let Some(mailbox) = self.backend.get_selected_mailbox() {
                        self.selected.insert(mailbox.id);
                    }
                }
                self.backend.select_next_mailbox();
            }
            Action::SelectPreviousMailbox => {
                if self.is_in_select_mode {
                    if let Some(mailbox) = self.backend.get_selected_mailbox() {
                        self.selected.insert(mailbox.id);
                    }
                }
                self.backend.select_previous_mailbox()
            }
            Action::SelectTopMailbox => self.backend.select_first_mailbox(),
            Action::SelectBottomMailbox => self.backend.select_last_mailbox(),
            Action::ActivateSelectedEntry => {
                if let Some(id) = self.backend.activate_selected_entry() {
                    self.app_actions.push(crate::Action::OpenRootMails(id));
                }
            }
            Action::ToggleMailbox => {
                if let Some(data) = self.backend.get_selected_mailbox() {
                    if !self.selected.remove(&data.id) {
                        self.selected.insert(data.id);
                    }
                    self.backend.select_next_mailbox();
                }
            }
            Action::EnterSelectMode => self.is_in_select_mode = true,
            Action::LeaveSelectMode => self.is_in_select_mode = false,
            Action::DiscardSelection => self.selected.clear(),

            Action::GoBack => self.backend.go_back(),
            Action::SetSortOrder => {
                if let Some(can_set_sort_order) = self.backend.can_set_sort_order() {
                    if can_set_sort_order {
                        let Some(data) = self.backend.get_selected_mailbox() else {
                            return;
                        };

                        self.overlay = Some(ScreenOverlay::Input(input::State::new(
                            "Set sort order (>= 0):",
                            InputType::SortOrder(data.id),
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
            Action::NormalizeSortOrder => self.backend.normalize_sort_order(),
            Action::CreateMailbox => {
                let parent = self.backend.get_parent_mailbox();

                self.overlay = Some(ScreenOverlay::Input(input::State::new(
                    "Create:",
                    InputType::NewMailboxName { parent },
                )));
            }
            Action::RemoveSelectedMailboxes => {
                if self.selected.is_empty() {
                    if let Some(data) = self.backend.get_selected_mailbox() {
                        self.backend.destroy_mailboxes(vec![data.id]);
                    }
                } else {
                    let ids = self.selected.drain().collect::<Vec<MailboxId>>();
                    self.backend.destroy_mailboxes(ids);
                }
            }
            Action::RenameSelectedMailboxes => {
                // TODO: Error handling
                let mailboxes = if self.selected.is_empty() {
                    self.backend
                        .get_selected_mailbox()
                        .map(|mailbox| vec![mailbox])
                } else {
                    let ids = self.selected.drain().collect::<Vec<MailboxId>>();
                    self.backend.get_mailboxes(&ids)
                };

                for mailbox in mailboxes.as_ref().unwrap().iter() {
                    tracing::debug!("{}", &mailbox.id);
                }

                if let Some(mailboxes) = mailboxes {
                    const RENAMING_PATH: &str = "/tmp/brieftaube-renaming.txt";
                    let file_content = mailboxes
                        .iter()
                        .map(|mailbox| format!("{}\n", mailbox.name))
                        .collect::<String>();
                    std::fs::write(RENAMING_PATH, file_content).expect("Create renaming file");

                    // open editor
                    {
                        ratatui::restore();
                        let status =
                            std::process::Command::new(self.backend.config.editor().unwrap())
                                .arg(RENAMING_PATH)
                                .status()
                                .unwrap();
                        if !status.success() {
                            error!("Couldn't start editor: {}", status);
                            return;
                        }
                        ratatui::init();
                    }

                    let new_names: Vec<String> = std::fs::read_to_string(RENAMING_PATH)
                        .unwrap()
                        .lines()
                        .map(|line| line.to_string())
                        .collect();

                    let mapping: Vec<(MailboxId, String)> = {
                        let ids = mailboxes.iter().map(|mailbox| mailbox.id.clone());
                        ids.zip(new_names.into_iter()).collect()
                    };

                    self.backend.rename_mailboxes(mapping);

                    self.app_actions.push(crate::Action::Redraw);
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
