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
    _account: Arc<backend::Fetcher>,

    mailboxes: Option<Vec<Mailbox>>,
}

impl State {
    pub fn new(account: Arc<backend::Fetcher>) -> Self {
        // let (tx, rx) = mpsc::channel(1);

        // let tx = Arc::new(tx);

        // let tx2 = tx.clone();
        // let account2 = account.clone();
        // tokio::spawn(async move {
        //     let mut response = {
        //         let mut request = account2.client.build();
        //         request
        //             .get_mailbox()
        //             .ids::<[_; 0], String>(None)
        //             .properties([
        //                 jmap_client::mailbox::Property::Id,
        //                 jmap_client::mailbox::Property::Name,
        //                 jmap_client::mailbox::Property::Role,
        //                 jmap_client::mailbox::Property::TotalEmails,
        //                 jmap_client::mailbox::Property::UnreadEmails,
        //             ]);
        //         request.send_get_mailbox().await.unwrap()
        //     };

        //     tx2.send(response.take_list()).await.unwrap();
        // });

        Self {
            app_actions: vec![],
            _account: account,
            mailboxes: None,
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

    pub fn get_mailboxes(&mut self) -> Option<Vec<Mailbox>> {
        self.mailboxes.clone()
    }

    pub fn get_mut_selected_mailbox(&mut self) -> Option<&mut Mailbox> {
        let Some(mailboxes) = &mut self.mailboxes else {
            return None;
        };

        let Some(idx) = self.mailbox_list_state.selected else {
            return None;
        };

        mailboxes.get_mut(idx)
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
                if let Some(selected_mailbox) = self.get_mut_selected_mailbox() {
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
