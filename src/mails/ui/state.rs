use crate::{
    backend,
    mails::ui::Action,
    utils::ui::{MailboxId, ScreenPalette, ScreenState, keybindmanager::KeybindManager, palette},
};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    palette: Option<palette::State<PaletteType>>,

    _fetcher: Arc<backend::Account>,

    /// `None`: Means that it's currently requested but the response didn't arrive yet.
    mails: HashMap<MailboxId, Vec<Email>>,

    selected_mailbox_id: Option<MailboxId>,
    list_state: tui_widget_list::ListState,
    keybindings: KeybindManager<Action>,

    mailbox_id: String,
}

impl State {
    pub fn new(fetcher: Arc<backend::Account>, id: MailboxId) -> Self {
        Self {
            app_actions: vec![],
            _fetcher: fetcher,
            palette: None,
            selected_mailbox_id: None,

            mails: HashMap::new(),
            mailbox_id: id,

            list_state: tui_widget_list::ListState::default(),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", Action::Quit.into()),
                (":", Action::OpenCommandPalette.into()),
                ("j", Action::SelectNextMail.into()),
                ("k", Action::SelectPreviousMail.into()),
                ("h", Action::OpenMailboxList.into()),
                ("l", Action::ViewSelectedMail.into()),
            ])),
        }
    }

    pub fn open_mailbox(&mut self, mailbox_id: Option<MailboxId>) {
        // if let Some(id) = mailbox_id {
        //     self.selected_mailbox_id = Some(id.clone());
        //     self.list_state.selected = None;

        //     let account = self.account.clone();
        //     let tx = self.tx.clone();
        //     tokio::spawn(async move {
        //         let client = &account.client;

        //         let initial_mails_ids = {
        //             let mut request = client.build();
        //             let query = request.query_email();
        //             query
        //                 .filter(jmap_client::email::query::Filter::in_mailbox(id))
        //                 .limit(100)
        //                 .calculate_total(true)
        //                 .position(0)
        //                 .sort([jmap_client::email::query::Comparator::received_at().descending()]);

        //             request.send_query_email().await.unwrap()
        //         };

        //         let mut mails = {
        //             let mut request = client.build();
        //             request
        //                 .get_email()
        //                 .ids(Some(initial_mails_ids.ids()))
        //                 .properties([
        //                     jmap_client::email::Property::Subject,
        //                     jmap_client::email::Property::From,
        //                     jmap_client::email::Property::ReceivedAt,
        //                     jmap_client::email::Property::Preview,
        //                     jmap_client::email::Property::ThreadId,
        //                 ]);

        //             request.send_get_email().await.unwrap()
        //         };

        //         tx.send(mails.take_list()).await.unwrap();
        //     });
        // }
        todo!()
    }

    pub fn get_render_mail_list_data(
        &mut self,
    ) -> Option<(&Vec<Email>, &mut tui_widget_list::ListState)> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                return Some((mails, &mut self.list_state));
            }
        }

        None
    }

    pub fn get_render_preview_data(&self) -> Option<&str> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                if let Some(idx) = self.list_state.selected {
                    return mails.get(idx).map(|mail| mail.preview().unwrap());
                }
            }
        }

        None
    }

    pub fn get_selected_mail(&self) -> Option<&Email> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                if let Some(idx) = self.list_state.selected {
                    return mails.get(idx);
                }
            }
        }

        None
    }

    pub fn select_next_mail(&mut self) {
        self.list_state.next();
    }

    pub fn select_previous_mail(&mut self) {
        self.list_state.previous();
    }
}

impl ScreenState<Action, PaletteType> for State {
    async fn update(&mut self) {
        // if let Some(selected_mailbox_id) = self.selected_mailbox_id.clone() {
        //     match self.rx.try_recv() {
        //         Ok(mails) => {
        //             self.mails.insert(selected_mailbox_id.to_string(), mails);
        //         }
        //         Err(mpsc::error::TryRecvError::Empty) => {}
        //         Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        //     }
        // }

        // if let Some(selected_mailbox_id) = self.selected_mailbox_id.as_ref() {
        //     let select_first_entry =
        //         self.mails.get(selected_mailbox_id).is_some() && self.list_state.selected.is_none();

        //     if select_first_entry {
        //         self.list_state.next();
        //     }
        // }

        false
    }

    fn apply_action(&mut self, action: Action) {
        tracing::debug!("Action: {:?}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),

            Action::SelectNextMail => self.select_next_mail(),
            Action::SelectPreviousMail => self.select_previous_mail(),

            Action::OpenMailboxList => {
                todo!();
            }
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(super::action::palette_options()));
            }
            Action::OpenLogs => {
                // return Some(super::Action::OpenLogs(Box::new(
                //     super::Action::OpenMailList(None),
                // )));
                todo!()
            }
            Action::CloseCommandPalette => self.palette = None,
            Action::ViewSelectedMail => {
                if let Some(_selected_mail) = self.get_selected_mail() {
                    todo!()
                    // return Some(super::Action::OpenMailViewer(Some(
                    //     selected_mail.id().unwrap().to_owned(),
                    // )));
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
