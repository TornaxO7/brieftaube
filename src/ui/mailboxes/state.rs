use super::Action;
use crate::{
    backend,
    ui::{
        ScreenPalette, ScreenState,
        utils::{keybindmanager::KeybindManager, palette},
    },
};
use jmap_client::mailbox::Mailbox;
use std::{collections::HashMap, sync::Arc};
use tracing::error;

#[derive(Debug, Clone)]
pub enum PaletteValues {}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    account: Arc<backend::Account>,

    pub mailboxes: Option<Vec<Mailbox>>,
    pub list_state: tui_widget_list::ListState,

    data_state: String,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        Self {
            app_actions: vec![],
            account,
            list_state: tui_widget_list::ListState::default(),

            mailboxes: None,
            data_state: String::new(),

            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit.into()),
                ("j", Action::SelectNextMailbox.into()),
                ("k", Action::SelectPreviousMailbox.into()),
                // ("n", super::Action::OpenComposer),
                ("<CR>", Action::OpenSelectedMailbox.into()),
                ("l", Action::OpenSelectedMailbox.into()),
            ])),
        }
    }
}

impl ScreenState<Action, PaletteValues> for State {
    fn update(&mut self) {
        if let Some((mailboxes, new_state)) = self.account.get_mailboxes(&self.data_state) {
            self.mailboxes = Some(mailboxes);
            self.data_state = new_state;
        }
    }

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => todo!(),
            Action::CloseCommandPalette => todo!(),

            Action::SelectNextMailbox => self.list_state.next(),
            Action::SelectPreviousMailbox => self.list_state.previous(),
            Action::OpenSelectedMailbox => {
                let Some(mailboxes) = self.mailboxes.as_ref() else {
                    error!("Can't open mailbox: There are no mailboxes available yet.");
                    return;
                };

                let Some(idx) = self.list_state.selected else {
                    error!("Can't open mailbox: No mailbox selected.");
                    return;
                };

                let mailbox_id = mailboxes[idx].id().unwrap().to_string();
                self.app_actions
                    .push(crate::Action::OpenRootMails(mailbox_id));
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

impl ScreenPalette<PaletteValues> for State {
    fn palette(&mut self) -> Option<&mut palette::State<PaletteValues>> {
        None
    }

    fn handle_palette_result(&mut self, _result: palette::HandleEventResult<PaletteValues>) {
        unreachable!("Huh")
    }
}
