use super::Action;
use crate::{
    backend::{self, mailboxes::MailboxData},
    ui::{
        ScreenPalette, ScreenState,
        utils::{keybindmanager::KeybindManager, palette},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tracing::error;

#[derive(Debug, Clone)]
pub enum PaletteValues {}

pub struct State {
    app_actions: Vec<crate::Action>,
    keybindings: KeybindManager<Action>,
    account: Arc<backend::Account>,

    pub mailboxes: Option<Vec<MailboxData>>,
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

    fn sort_mailboxes(&self, mailboxes: &[MailboxData]) {
        if !mailboxes.is_empty() {
            todo!()
        }
    }
}

impl ScreenState<Action, PaletteValues> for State {
    fn update(&mut self) {
        if let Some((mailboxes, new_state)) = self.account.get_mailboxes(&self.data_state) {
            if self.list_state.selected.is_none() && !mailboxes.is_empty() {
                self.list_state.selected = Some(0);
            }

            // let are_unsorted = {
            //     let mut used_order_numbers = HashSet::with_capacity(mailboxes.len());
            //     for mailbox in mailboxes.iter() {
            //         used_order_numbers.insert(mailbox.sort_order);
            //     }
            //     used_order_numbers.len() < mailboxes.len()
            // };

            // if are_unsorted {
            //     self.sort_mailboxes(&mut mailboxes);
            // }

            // for mailbox in mailboxes.iter() {
            //     tracing::debug!("{}", mailbox.sort_order);
            // }

            // mailboxes.sort_by(|a, b| a.sort_order().cmp(&b.sort_order()));

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
            Action::OpenSelectedMailbox => match self.get_selected_mailbox() {
                Ok(mailbox) => {
                    self.app_actions
                        .push(crate::Action::OpenRootMails(mailbox.id.clone()));
                }
                Err(err) => error!(err),
            },
            Action::SetSortOrder => {
                todo!()
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
