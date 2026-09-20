mod column;
mod message;
mod message_request;
mod user_action;
mod view;

mod mailbox_column;
mod thread_column;
mod user_column;

use crate::{
    config::{self, Username},
    types::{
        AccountData, AccountId, MailKeyword, MailboxData, ParentMailboxId, ROOT_MAILBOX_ID,
        ThreadId,
    },
    ui::{
        Layer, Loadable,
        mailfs::{
            mailbox_column::MailboxColumn,
            thread_column::ThreadColumn,
            user_column::{UserColumn, UserColumnEntryMut},
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
    mailbox_columns: HashMap<(Username, AccountId, ParentMailboxId), MailboxColumn>,
    thread_columns: HashMap<(Username, AccountId, ThreadId), HashMap<ThreadId, ThreadColumn>>,
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
            Message::SetChildMailboxes {
                username,
                account_id,
                parent_id,
                child_mailboxes,
            } => self.handle_set_child_mailboxes(username, account_id, parent_id, child_mailboxes),
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

    fn handle_set_child_mailboxes(
        &mut self,
        username: Username,
        account_id: AccountId,
        parent_id: ParentMailboxId,
        child_mailboxes: color_eyre::Result<Vec<MailboxData>>,
    ) -> Vec<super::Message> {
        let key = (username, account_id, parent_id);

        let mailbox_column = self.mailbox_columns.get_mut(&key).unwrap();

        mailbox_column.mailboxes = match child_mailboxes {
            Ok(mailboxes) => Loadable::Loaded(mailboxes),
            Err(err) => Loadable::Error(err.to_string()),
        };

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
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_down();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let selected_account = self.users_column.get_selected_account().unwrap();
                let key = selected_account.as_key(mailbox_id);

                let mailbox_column = self.mailbox_columns.get_mut(&key).unwrap();
                mailbox_column.navigate_down();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Thread(_thread_id) => {
                todo!()
            }
        }
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_up();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let selected_account = self.users_column.get_selected_account().unwrap();
                let key = selected_account.as_key(mailbox_id);

                let mailbox_column = self.mailbox_columns.get_mut(&key).unwrap();
                mailbox_column.navigate_up();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Thread(_thread_id) => {
                todo!();
            }
        }
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_to_top();
                vec![]
            }
            ColumnStackEntry::Mailbox(_mailbox_id) => {
                todo!()
            }
            ColumnStackEntry::Thread(_thread_id) => {
                todo!();
            }
        }
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_to_bottom();
                vec![]
            }
            ColumnStackEntry::Mailbox(_mailbox_id) => {
                todo!();
            }
            ColumnStackEntry::Thread(_thread_id) => {
                todo!();
            }
        }
    }

    fn navigate_right(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().cloned().unwrap() {
            ColumnStackEntry::Users => {
                let Some(selected_entry) = self.users_column.get_selected_entry_mut() else {
                    return vec![];
                };

                match selected_entry {
                    UserColumnEntryMut::User(user_ctx) => {
                        if user_ctx.is_collapsed {
                            user_ctx.is_collapsed = false;
                        }

                        if matches!(user_ctx.accounts, Loadable::NotLoaded | Loadable::Error(_)) {
                            user_ctx.accounts = Loadable::Loading;
                            return vec![
                                MessageRequest::GetAccountsOf(user_ctx.config.clone()).into(),
                            ];
                        }

                        vec![]
                    }
                    UserColumnEntryMut::Account(_account_data) => {
                        self.column_stack
                            .push(ColumnStackEntry::Mailbox(ROOT_MAILBOX_ID));

                        vec![]
                    }
                    UserColumnEntryMut::AccountNotLoaded
                    | UserColumnEntryMut::AccountLoading
                    | UserColumnEntryMut::AccountError(_) => vec![],
                }
            }
            ColumnStackEntry::Mailbox(_mailbox_id) => {
                todo!();
            }
            ColumnStackEntry::Thread(_thread_id) => {
                todo!()
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

// helpers
impl State {
    fn ensure_right_column_data(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                let Some(account_key) = self.users_column.get_selected_account() else {
                    return vec![];
                };

                let key = account_key.as_key(ROOT_MAILBOX_ID);

                if self.mailbox_columns.contains_key(&key) {
                    vec![]
                } else {
                    self.mailbox_columns
                        .insert(key.clone(), MailboxColumn::new_root());

                    vec![
                        MessageRequest::GetChildMailboxes {
                            username: key.0,
                            account_id: key.1,
                            parent_id: key.2,
                        }
                        .into(),
                    ]
                }
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let Some(account_key) = self.users_column.get_selected_account() else {
                    return vec![];
                };

                let key = account_key.as_key(mailbox_id);

                if self.mailbox_columns.contains_key(&key) {
                    vec![]
                } else {
                    self.mailbox_columns
                        .insert(key.clone(), MailboxColumn::new_root());

                    vec![
                        MessageRequest::GetChildMailboxes {
                            username: key.0,
                            account_id: key.1,
                            parent_id: key.2,
                        }
                        .into(),
                    ]
                }
            }
            ColumnStackEntry::Thread(_thread_id) => todo!(),
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
    fn navigate_up(&mut self);

    fn navigate_down(&mut self);

    fn navigate_to_bottom(&mut self);

    fn navigate_to_top(&mut self);
}

#[derive(Clone, Hash)]
struct AccountKey {
    username: Username,
    account_id: AccountId,
}

impl AccountKey {
    pub fn as_key<T>(&self, other: T) -> (Username, AccountId, T) {
        (self.username.clone(), self.account_id.clone(), other)
    }
}
