use super::Action;
use crate::{
    backend::{Account, mailboxes::MailboxData},
    ui::{
        ScreenOverlay, ScreenOverlayResult, ScreenState,
        utils::{self, keybindmanager::KeybindManager},
    },
};
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

    pub mailboxes: Option<Vec<MailboxData>>,
    pub list_state: tui_widget_list::ListState,

    data_state: String,
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            app_actions: vec![],
            account,
            list_state: tui_widget_list::ListState::default(),

            mailboxes: None,
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

    fn get_selected_mailbox(&self) -> Result<&MailboxData, &'static str> {
        let mailboxes = self
            .mailboxes
            .as_ref()
            .ok_or("Can't get selected mailbox: There are no mailboxes available yet.")?;
        let idx = self
            .list_state
            .selected
            .ok_or("Can't get selected mailbox: No mailbox selected.")?;

        Ok(&mailboxes[idx])
    }
}

impl ScreenState<Action, PaletteValue, InputType> for State {
    #[tracing::instrument(level = "debug", skip_all)]
    fn update(&mut self) {
        if let Some((mut mailboxes, new_state)) = self.account.get_mailboxes(&self.data_state) {
            trace!("Updating");
            if self.list_state.selected.is_none() && !mailboxes.is_empty() {
                self.list_state.selected = Some(0);
            }

            mailboxes.sort_unstable_by_key(|mailbox| mailbox.sort_order);

            self.mailboxes = Some(mailboxes);
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

            Action::SelectNextMailbox => self.list_state.next(),
            Action::SelectPreviousMailbox => self.list_state.previous(),
            Action::OpenSelectedMailbox => match self.get_selected_mailbox() {
                Ok(mailbox) => {
                    self.app_actions
                        .push(crate::Action::OpenRootMails(mailbox.id.clone()));
                }
                Err(err) => error!(err),
            },
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
            ScreenOverlayResult::Input { value, typ } => {
                todo!()
            }
            ScreenOverlayResult::Cancel => {}
        }
    }
}
