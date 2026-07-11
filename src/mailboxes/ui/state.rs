use super::Action;
use crate::{
    backend,
    utils::ui::{ScreenPalette, ScreenState, keybindmanager::KeybindManager, palette},
};
use jmap_client::mailbox::Mailbox;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub enum PaletteValues {}

pub struct State {
    app_actions: Vec<crate::Action>,
    mailbox_list_state: tui_widget_list::ListState,
    keybindings: KeybindManager<Action>,
    account: Arc<backend::Account>,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        Self {
            app_actions: vec![],
            account,
            mailbox_list_state: tui_widget_list::ListState::default(),

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

    pub fn get_mailboxes(&mut self) -> Option<&[Mailbox]> {
        todo!()
    }

    pub fn get_selected_mailbox(&self) -> Option<&Mailbox> {
        let Some(mailboxes) = self.get_mailboxes() else {
            return None;
        };

        let Some(idx) = self.mailbox_list_state.selected else {
            return None;
        };

        mailboxes.get(idx)
    }

    pub fn get_list_state(&mut self) -> &mut tui_widget_list::ListState {
        &mut self.mailbox_list_state
    }

    pub fn select_next_mailbox(&mut self) {
        self.mailbox_list_state.next();
    }

    pub fn select_previous_mailbox(&mut self) {
        self.mailbox_list_state.previous();
    }
}

impl ScreenState<Action, PaletteValues> for State {
    async fn update(&mut self) -> bool {
        // match self.rx.try_recv() {
        //     Ok(mailboxes) => self.mailboxes = Some(mailboxes),
        //     Err(mpsc::error::TryRecvError::Empty) => {}
        //     Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        // }

        // let select_first_entry =
        //     self.mailboxes.is_some() && self.mailbox_list_state.selected.is_none();

        // if select_first_entry {
        //     self.mailbox_list_state.next();
        // }
        false
    }

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => todo!(),
            Action::CloseCommandPalette => todo!(),

            Action::SelectNextMailbox => self.select_next_mailbox(),
            Action::SelectPreviousMailbox => self.select_previous_mailbox(),
            Action::OpenSelectedMailbox => {
                if let Some(selected_mailbox) = self.get_selected_mailbox() {
                    assert!(selected_mailbox.id().is_some());
                    // return Some(super::Action::OpenMailList(Some(
                    //     selected_mailbox.id().unwrap().to_string(),
                    // )));
                    todo!("Open mail list command")
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

impl ScreenPalette<PaletteValues> for State {
    fn palette(&mut self) -> Option<&mut palette::State<PaletteValues>> {
        None
    }

    fn handle_palette_result(&mut self, _result: palette::HandleEventResult<PaletteValues>) {
        unreachable!("Huh")
    }
}
