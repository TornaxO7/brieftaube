use super::Action;
use crate::{
    backend,
    ui::{
        ScreenPalette, ScreenState, ThreadId,
        utils::{keybindmanager::KeybindManager, palette},
    },
};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tracing::error;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<Action>,
    account: Arc<backend::Account>,
    thread_id: String,

    pub mails: Option<Vec<Email>>,
    pub list_state: tui_widget_list::ListState,
    mails_state: String,
}

impl State {
    pub fn new(account: Arc<backend::Account>, thread_id: ThreadId) -> Self {
        Self {
            app_actions: vec![],
            palette: None,
            account,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("j", Action::SelectNextMail),
                ("k", Action::SelectPreviousMail),
                ("h", Action::Back),
                ("l", Action::ViewSelectedMail),
            ])),

            mails: None,
            mails_state: String::new(),
            thread_id,
            list_state: tui_widget_list::ListState::default(),
        }
    }

    fn select_next_mail(&mut self) {
        self.list_state.next();
    }

    fn select_previous_mail(&mut self) {
        self.list_state.previous();
    }

    fn get_selected_mail(&self) -> Option<&Email> {
        let Some(mails) = self.mails.as_ref() else {
            error!("Can't get seleced mail: Mails aren't available yet to select.");
            return None;
        };

        let Some(idx) = self.list_state.selected else {
            error!("Can't get selected mail: No mail is selected.");
            return None;
        };

        Some(&mails[idx])
    }
}

impl ScreenState<Action, PaletteType> for State {
    fn update(&mut self) {
        if let Some((mails, new_state)) = self
            .account
            .get_thread_mails(&self.thread_id, &self.mails_state)
        {
            if self.list_state.selected.is_none() && !mails.is_empty() {
                self.list_state.selected = Some(0);
            }

            self.mails = Some(mails);
            self.mails_state = new_state;
        }
    }

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::Back => self.app_actions.push(crate::Action::Back),

            Action::SelectNextMail => self.select_next_mail(),
            Action::SelectPreviousMail => self.select_previous_mail(),

            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(super::action::palette_options()));
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::CloseCommandPalette => self.palette = None,
            Action::ViewSelectedMail => {
                if let Some(mail) = self.get_selected_mail() {
                    self.app_actions
                        .push(crate::Action::OpenMailViewer(mail.clone()));
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
}

impl ScreenPalette<PaletteType> for State {
    fn palette(&mut self) -> Option<&mut palette::State<PaletteType>> {
        self.palette.as_mut()
    }

    fn handle_palette_result(&mut self, result: palette::HandleEventResult<PaletteType>) {
        self.palette = None;

        match result {
            palette::HandleEventResult::Cancel => {}
            palette::HandleEventResult::Selected(value) => match value {
                PaletteType::Action(action) => {
                    self.apply_action(action);
                }
            },
        }
    }
}
