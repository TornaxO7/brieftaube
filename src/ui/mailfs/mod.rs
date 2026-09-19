mod column;
mod message;
mod message_request;
mod user_action;
mod view;

mod mailbox_column;
mod thread_column;
mod user_column;

use crate::{
    config,
    datasource::types::QueryWindow,
    types::{AccountData, MailKeyword, ParentMailboxId, ThreadId},
    ui::{
        Layer, Loadable,
        mailfs::{
            mailbox_column::MailboxColumn, thread_column::ThreadColumn, user_column::UserColumn,
        },
        utils::keybindmanager::{self, KeybindManager},
    },
};
use crossterm::event::Event;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::*;
pub use message_request::*;
pub use view::view;

pub struct State {
    keybindings: KeybindManager<UserAction>,

    throbber: ThrobberState,
    mode: Mode,

    column_stack: Vec<ColumnStackEntry>,

    users_column: UserColumn,
    mailbox_columns: HashMap<ParentMailboxId, MailboxColumn>,
    thread_columns: HashMap<ThreadId, ThreadColumn>,
}

impl State {
    pub fn new() -> Self {
        let users_column = UserColumn::new();
        let thread_columns = HashMap::new();
        let mailbox_columns = HashMap::new();

        Self {
            throbber: ThrobberState::default(),
            mode: Mode::Normal,
            column_stack: vec![ColumnStackEntry::Users],

            thread_columns,
            users_column,
            mailbox_columns,

            keybindings: KeybindManager::new(HashMap::from([
                ("q", UserAction::Quit),
                ("j", UserAction::NavigateDown),
                ("l", UserAction::NavigateRight),
                ("h", UserAction::NavigateLeft),
                ("k", UserAction::NavigateUp),
                ("gg", UserAction::NavigateToTop),
                ("ge", UserAction::NavigateToBottom),
                (" ", UserAction::SelectEntryToggle),
                (":", UserAction::OpenCommandPalette),
            ])),
        }
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        self.throbber.calc_next();
        match msg {
            Message::Event(event) => self.handle_event(event),
            Message::UserAction(action) => self.handle_user_action(action),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
            Message::SetUserAccounts {
                username: to,
                accounts,
            } => self.handle_set_user_accounts(to, accounts),
        }
    }
}

impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::Mouse(_)
            | Event::Paste(_)
            | Event::Resize(_, _)
            | Event::FocusGained
            | Event::FocusLost => vec![],
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                keybindmanager::HandleEvent::Action(action) => self.handle_user_action(action),
                keybindmanager::HandleEvent::Registered => vec![],
                keybindmanager::HandleEvent::Cancel => vec![],
            },
        }
    }

    fn handle_user_action(&mut self, action: UserAction) -> Vec<super::Message> {
        debug!("{:?}", action);

        match action {
            UserAction::Quit => self.quit(),
            UserAction::OpenCommandPalette => self.open_command_palette(),
            UserAction::NavigateDown => self.navigate_down(),
            UserAction::NavigateUp => self.navigate_up(),
            UserAction::NavigateToTop => self.navigate_to_top(),
            UserAction::NavigateToBottom => self.navigate_to_bottom(),
            UserAction::NavigateRight => self.navigate_right(),
            UserAction::NavigateLeft => self.navigate_left(),
            UserAction::NavigateToParent => self.navigate_to_parent(),

            UserAction::SelectEntryToggle => self.select_entry(),
            UserAction::CutSelectedEntries => self.cut_selected_entries(),
            UserAction::PasteSelectedEntries => self.paste_selected_entries(),

            UserAction::MoveMailboxUp => self.move_mailbox_up(),
            UserAction::MoveMailboxDown => self.move_mailbox_down(),

            UserAction::CreateMailbox => self.create_mailbox(),
            UserAction::RemoveMailbox => self.remove_mailbox(),
            // UserAction::MarkMailAsUnseen => self.mail_patch_keywords(&[(MailKeyword::Seen, false)]),
            // UserAction::MarkMailAsSeen => self.mail_patch_keywords(&[(MailKeyword::Seen, true)]),
        }
    }

    fn handle_selected_palette_entry(&mut self, entry: String) -> Vec<super::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        vec![super::Message::Mailfs(Message::UserAction(action))]
    }

    fn handle_set_user_accounts(
        &mut self,
        username: config::Username,
        accounts: Loadable<Vec<AccountData>>,
    ) -> Vec<super::Message> {
        self.users_column.set_accounts(username, accounts);
        vec![]
    }
}

/// Action implementations
impl State {
    fn quit(&self) -> Vec<super::Message> {
        vec![super::Message::Quit]
    }

    fn open_command_palette(&self) -> Vec<super::Message> {
        let entries = UserAction::palette_options();
        vec![super::Message::OpenPalette {
            entries,
            map: |entry| super::Message::Mailfs(Message::SelectedPaletteEntry(entry)),
        }]
    }

    fn navigate_down(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_down(),
            ColumnStackEntry::Mailbox(mailbox_id) => self
                .mailbox_columns
                .get_mut(mailbox_id)
                .unwrap()
                .navigate_down(),
            ColumnStackEntry::Thread(thread_id) => self
                .thread_columns
                .get_mut(&thread_id)
                .unwrap()
                .navigate_down(),
        }
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_up(),
            ColumnStackEntry::Mailbox(mailbox_id) => self
                .mailbox_columns
                .get_mut(mailbox_id)
                .unwrap()
                .navigate_up(),
            ColumnStackEntry::Thread(thread_id) => self
                .thread_columns
                .get_mut(thread_id)
                .unwrap()
                .navigate_up(),
        }
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_to_top(),
            ColumnStackEntry::Mailbox(mailbox_id) => self
                .mailbox_columns
                .get_mut(mailbox_id)
                .unwrap()
                .navigate_to_top(),
            ColumnStackEntry::Thread(thread_id) => self
                .thread_columns
                .get_mut(thread_id)
                .unwrap()
                .navigate_to_top(),
        }
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_to_bottom(),
            ColumnStackEntry::Mailbox(mailbox_id) => self
                .mailbox_columns
                .get_mut(mailbox_id)
                .unwrap()
                .navigate_to_bottom(),
            ColumnStackEntry::Thread(thread_id) => self
                .thread_columns
                .get_mut(thread_id)
                .unwrap()
                .navigate_to_bottom(),
        }
    }

    fn navigate_right(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().cloned().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_right(),
            ColumnStackEntry::Mailbox(mailbox_id) => {
                self.mailbox_columns
                    .entry(mailbox_id.clone())
                    .or_insert(MailboxColumn::loading());

                let account_id = self.users_column.get_selected_account().unwrap().id.clone();

                let mut request_messages: Vec<super::Message> = vec![
                    MessageRequest::GetChildMailboxes {
                        account_id: account_id.clone(),
                        parent: mailbox_id.clone(),
                    }
                    .into(),
                ];

                if let Some(id) = mailbox_id.clone() {
                    request_messages.push(
                        MessageRequest::QueryMails {
                            account_id: account_id.clone(),
                            mailbox: id,
                            window: QueryWindow {
                                start: 0,
                                limit: 30,
                            },
                        }
                        .into(),
                    );
                }

                request_messages
            }
            ColumnStackEntry::Thread(thread_id) => {
                self.column_stack
                    .push(ColumnStackEntry::Thread(thread_id.clone()));

                match self
                    .thread_columns
                    .get_mut(&thread_id)
                    .map(|thread_column| &thread_column.mails)
                {
                    Some(Loadable::NotLoaded) | Some(Loadable::Error(_)) | None => {
                        self.thread_columns
                            .insert(thread_id.clone(), ThreadColumn::loading());

                        let account_id =
                            self.users_column.get_selected_account().unwrap().id.clone();

                        vec![
                            MessageRequest::GetThreadMails {
                                account_id: account_id,
                                thread: thread_id,
                            }
                            .into(),
                        ]
                    }
                    Some(Loadable::Loading) | Some(Loadable::Loaded(_)) => {
                        vec![]
                    }
                }
            }
        }
    }

    fn navigate_left(&mut self) -> Vec<super::Message> {
        match self.column_stack.last_mut().unwrap() {
            ColumnStackEntry::Users => self.users_column.navigate_left(),
            ColumnStackEntry::Mailbox(_) | ColumnStackEntry::Thread(_) => {
                self.column_stack.pop();
                vec![]
            }
        }
    }

    fn navigate_to_parent(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn select_entry(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn cut_selected_entries(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn paste_selected_entries(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn move_mailbox_up(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn move_mailbox_down(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn create_mailbox(&mut self) -> Vec<super::Message> {
        todo!();
        // Some(super::Message::OpenPrompt {
        //     description: "Mailbox name:".to_string(),
        //     map: |entry| super::Message::Mailfs(Message::SelectedPaletteEntry(entry)),
        // })
    }

    fn remove_mailbox(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn mail_patch_keywords(&mut self, _patch: &[(MailKeyword, bool)]) -> Vec<super::Message> {
        todo!();
    }
}

struct UserData {
    user: config::UserConfig,
    is_collapsed: bool,
    accounts: Loadable<Vec<AccountData>>,
}

impl UserData {
    fn new(user_config: config::UserConfig) -> Self {
        Self {
            user: user_config,
            is_collapsed: true,
            accounts: Loadable::NotLoaded,
        }
    }

    fn len(&self) -> usize {
        if self.is_collapsed {
            return 1;
        }

        match &self.accounts {
            Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => 1,
            Loadable::Loaded(accounts) => accounts.len() + 1,
        }
    }
}

#[derive(strum::Display, Debug, Clone, Copy)]
enum Mode {
    Normal,
}

#[derive(Debug, Clone, Hash)]
enum ColumnStackEntry {
    Users,
    Mailbox(ParentMailboxId),
    Thread(ThreadId),
}

trait MailfsColumn {
    fn navigate_up(&mut self) -> Vec<super::Message>;

    fn navigate_down(&mut self) -> Vec<super::Message>;

    fn navigate_to_bottom(&mut self) -> Vec<super::Message>;

    fn navigate_to_top(&mut self) -> Vec<super::Message>;
}
