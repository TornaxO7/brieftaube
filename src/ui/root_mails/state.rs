use super::Action;
use crate::{
    backend,
    ui::{
        MailboxId, ScreenOverlay, ScreenOverlayResult, ScreenState,
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
    keybindings: KeybindManager<Action>,
    account: Arc<backend::Account>,
    mailbox_id: String,
    overlay: Option<ScreenOverlay<PaletteType>>,

    pub root_mails: Option<Vec<Email>>,
    pub list_state: tui_widget_list::ListState,
    mails_state: String,
}

impl State {
    pub fn new(fetcher: Arc<backend::Account>, id: MailboxId) -> Self {
        Self {
            app_actions: vec![],
            overlay: None,
            account: fetcher,
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit),
                (":", Action::OpenCommandPalette),
                ("j", Action::SelectNextMail),
                ("k", Action::SelectPreviousMail),
                ("h", Action::Back),
                ("l", Action::OpenThread),
            ])),

            root_mails: None,
            mails_state: String::new(),
            mailbox_id: id,
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
        let Some(mails) = self.root_mails.as_ref() else {
            error!("Can't get selected mail: Mails aren't available yet.");
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
        if let Some((root_mails, new_state)) = self
            .account
            .get_root_mails(&self.mailbox_id, &self.mails_state)
        {
            if self.list_state.selected.is_none() && !root_mails.is_empty() {
                self.list_state.selected = Some(0);
            }

            self.root_mails = Some(root_mails);
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
                self.overlay = Some(ScreenOverlay::Palette(palette::State::new(
                    super::action::palette_options(),
                )));
            }
            Action::OpenLogs => {
                self.app_actions.push(crate::Action::OpenLogViewer);
            }
            Action::OpenThread => {
                if let Some(selected_mail) = self.get_selected_mail() {
                    let thread_id = selected_mail.thread_id().unwrap().to_string();
                    self.app_actions.push(crate::Action::OpenThread(thread_id));
                }
            }
            Action::ViewSelectedMail => {
                if let Some(selected_mail) = self.get_selected_mail() {
                    self.app_actions
                        .push(crate::Action::OpenMailViewer(selected_mail.clone()));
                }
            }
            Action::ComposeMail => {
                self.app_actions.push(crate::Action::OpenComposer);
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }

    fn overlay(&mut self) -> Option<&mut crate::ui::ScreenOverlay<PaletteType>> {
        self.overlay.as_mut()
    }

    fn handle_overlay_result(&mut self, result: crate::ui::ScreenOverlayResult<PaletteType>) {
        self.overlay = None;

        match result {
            ScreenOverlayResult::Cancel => {}
            ScreenOverlayResult::Palette(value) => match value {
                PaletteType::Action(action) => {
                    self.apply_action(action);
                }
            },
            ScreenOverlayResult::Input(_) => {
                unreachable!("Sus")
            }
        }
    }
}
