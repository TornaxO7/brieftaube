mod columns;
mod message;
mod message_request;
mod user_action;
mod view;

use crate::{
    config::{self, UserConfig, Username},
    datasource::types::QueryWindow,
    types::{
        AccountData, AccountId, MailDataCore, MailDataPreview, MailId, MailboxData, MailboxId,
        ParentMailboxId, ROOT_MAILBOX_ID, ThreadId,
    },
    ui::{
        Layer, Loadable,
        mailfs::columns::*,
        utils::keybindmanager::{self, KeybindManager},
    },
};
use crossterm::event::Event;
use ratatui::layout::Rect;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::*;
pub use message_request::*;
pub use view::view;

// TODO: split up `User & Accounts` into `User` column and `Accounts` column
pub struct State {
    keybindings: KeybindManager<UserAction>,

    throbber: ThrobberState,
    mode: Mode,

    column_stack: Vec<ColumnStackEntry>,

    terminal_height: u16,

    users_column: columns::UserColumn,
    accounts_column: HashMap<Username, Loadable<columns::AccountsColumn>>,
    mailbox_columns: HashMap<(Username, AccountId, ParentMailboxId), columns::MailboxColumn>,
    thread_columns: HashMap<(Username, AccountId, ThreadId), Loadable<columns::ThreadColumn>>,
    mail_previews: HashMap<(Username, AccountId, MailId), Loadable<MailDataPreview>>,
}

impl State {
    pub fn new(init_rect: Rect) -> (Self, UserConfig) {
        let users_column = columns::UserColumn::new();
        let (accounts_column, initial_user) = {
            let selected_user = users_column.get_selected_entry().clone();
            (
                HashMap::from([(selected_user.username.clone(), Loadable::Loading)]),
                selected_user,
            )
        };
        let thread_columns = HashMap::new();
        let mailbox_columns = HashMap::new();
        let mail_previews = HashMap::new();

        (
            Self {
                throbber: ThrobberState::default(),
                mode: Mode::Normal,
                column_stack: vec![ColumnStackEntry::Users],
                terminal_height: init_rect.height,

                accounts_column,
                thread_columns,
                users_column,
                mailbox_columns,
                mail_previews,

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
            },
            initial_user,
        )
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        self.throbber.calc_next();
        let mut response_msgs = match msg {
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
            Message::SetMails {
                username,
                account_id,
                mailbox,
                window,
                result,
            } => self.handle_set_mails(username, account_id, mailbox, window, result),
            Message::SetThreadMails {
                username,
                account_id,
                thread_id,
                thread_mails,
            } => self.handle_set_thread_mails(username, account_id, thread_id, thread_mails),
            Message::SetMailPreview {
                username,
                account_id,
                mail_id,
                preview,
            } => self.handle_set_mail_preview(username, account_id, mail_id, preview),
        };

        response_msgs.extend(self.ensure_right_column_data());

        response_msgs
    }
}

impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::Mouse(_) | Event::Paste(_) | Event::FocusGained | Event::FocusLost => vec![],
            Event::Resize(_, new_height) => {
                self.terminal_height = new_height;

                // TODO: If new height exceeds mails list for mailbox column => query more
                vec![]
            }
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
        accounts: color_eyre::Result<Vec<AccountData>>,
    ) -> Vec<super::Message> {
        match accounts {
            Ok(accounts) => {
                let res = self.accounts_column.insert(
                    username,
                    Loadable::Loaded(columns::AccountsColumn::new(accounts)),
                );
                debug_assert!(res.is_some());
            }
            Err(err) => {
                self.accounts_column
                    .insert(username, Loadable::Error(err.to_string()));
            }
        }
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
        mailbox_column.set_mailboxes(child_mailboxes);
        vec![]
    }

    fn handle_set_mails(
        &mut self,
        username: Username,
        account_id: AccountId,
        mailbox_id: MailboxId,
        window: QueryWindow,
        result: color_eyre::Result<(Vec<MailDataCore>, Option<usize>)>,
    ) -> Vec<super::Message> {
        let key = (username, account_id, Some(mailbox_id.clone()));

        let column = self
            .mailbox_columns
            .get_mut(&key)
            .expect("The mailbox column itself should request this so it must be there.");
        column.set_mails(window, result);
        vec![]
    }

    fn handle_set_thread_mails(
        &mut self,
        username: Username,
        account_id: AccountId,
        thread_id: ThreadId,
        thread_mails: color_eyre::Result<Vec<MailDataCore>>,
    ) -> Vec<super::Message> {
        let key = (username, account_id, thread_id);

        match thread_mails {
            Ok(mails) => {
                let res = self
                    .thread_columns
                    .insert(key, Loadable::Loaded(ThreadColumn::new(mails)));
                debug_assert!(res.is_some());
            }
            Err(err) => {
                self.thread_columns
                    .insert(key, Loadable::Error(err.to_string()));
            }
        }
        vec![]
    }

    fn handle_set_mail_preview(
        &mut self,
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
        mail_preview: color_eyre::Result<MailDataPreview>,
    ) -> Vec<super::Message> {
        let key = (username, account_id, mail_id);

        let current_preview = self.mail_previews.get_mut(&key).unwrap();

        *current_preview = match mail_preview {
            Ok(preview) => Loadable::Loaded(preview),
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
            ColumnStackEntry::Accounts => {
                let username = &self.users_column.get_selected_entry().username;
                let Some(column) = self
                    .accounts_column
                    .get_mut(username)
                    .expect("Account column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_down();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id);
                let column = self.mailbox_columns.get_mut(&key).unwrap();
                column.navigate_down();
                // TODO: Check if the query-window is still within the new height
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id);
                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_down();
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_up();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Accounts => {
                let username = &self.users_column.get_selected_entry().username;
                let Some(column) = self
                    .accounts_column
                    .get_mut(username)
                    .expect("Account column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_up();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id);
                let mailbox_column = self.mailbox_columns.get_mut(&key).unwrap();
                mailbox_column.navigate_up();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id);
                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Colum exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_up();
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_to_top();
                vec![]
            }
            ColumnStackEntry::Accounts => {
                let selected_user = &self.users_column.get_selected_entry().username;
                let Some(column) = self
                    .accounts_column
                    .get_mut(selected_user)
                    .expect("Account column exist")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_to_top();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                self.mailbox_columns
                    .get_mut(&key)
                    .unwrap()
                    .navigate_to_top();
                vec![]
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id.clone());
                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_to_top();
                vec![]
            }
        }
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_to_bottom();
                vec![]
            }
            ColumnStackEntry::Accounts => {
                let selected_user = &self.users_column.get_selected_entry().username;
                let Some(column) = self
                    .accounts_column
                    .get_mut(selected_user)
                    .expect("Accounts column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };
                column.navigate_to_bottom();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                self.mailbox_columns
                    .get_mut(&key)
                    .unwrap()
                    .navigate_to_bottom();
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id.clone());
                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_to_bottom();
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_right(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().cloned().unwrap() {
            ColumnStackEntry::Users => {
                let selected_user = self.users_column.get_selected_entry();

                match self
                    .accounts_column
                    .get(&selected_user.username)
                    .expect("Accounts column exists")
                {
                    Loadable::Loading => vec![],
                    Loadable::NotLoaded | Loadable::Error(_) => {
                        // retry
                        vec![MessageRequest::GetAccountsOf(selected_user.clone()).into()]
                    }
                    Loadable::Loaded(_) => {
                        self.column_stack.push(ColumnStackEntry::Accounts);
                        self.ensure_right_column_data()
                    }
                }
            }
            ColumnStackEntry::Accounts => {
                self.column_stack
                    .push(ColumnStackEntry::Mailbox(ROOT_MAILBOX_ID));
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id);
                let column = self.mailbox_columns.get(&key).unwrap();
                let Some(selected_entry) = column.get_selected_entry() else {
                    // mailbox could be empty
                    return vec![];
                };

                match selected_entry {
                    Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                        vec![]
                    }
                    Loadable::Loaded(entry) => {
                        match entry {
                            MailboxColumnEntry::Mailbox(mailbox_data) => {
                                self.column_stack
                                    .push(ColumnStackEntry::Mailbox(Some(mailbox_data.id.clone())));
                            }
                            MailboxColumnEntry::RootMail(root_mail) => {
                                self.column_stack
                                    .push(ColumnStackEntry::Thread(root_mail.thread_id.clone()));
                            }
                        };

                        self.ensure_right_column_data()
                    }
                }
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id);

                match self.thread_columns.get(&key).expect("Column exists") {
                    Loadable::NotLoaded => unreachable!("Start loading?"),
                    Loadable::Loading => vec![],
                    Loadable::Error(_) => {
                        unreachable!("Middle column can't be this thread if it's an error")
                    }
                    Loadable::Loaded(column) => {
                        let selected_mail = column.get_selected_entry();
                        vec![super::Message::OpenReader {
                            username: key.0,
                            account_id: key.1,
                            mail_id: selected_mail.id.clone(),
                        }]
                    }
                }
            }
        }
    }

    fn navigate_left(&mut self) -> Vec<super::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                vec![]
            }
            ColumnStackEntry::Accounts
            | ColumnStackEntry::Mailbox(_)
            | ColumnStackEntry::Thread(_) => {
                self.column_stack.pop();
                vec![]
            }
        }
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
}

// helpers
impl State {
    fn get_account_ctx(&self) -> AccountCtx {
        let selected_username = self.users_column.get_selected_entry().username.clone();
        let selected_account_id = self
            .accounts_column
            .get(&selected_username)
            .expect("Account column exists")
            .loaded()
            .expect("Account column is loaded")
            .get_selected_entry()
            .id
            .clone();

        AccountCtx {
            username: selected_username,
            account_id: selected_account_id,
        }
    }

    /// Depending on what is selected in the middle column it will return the suitable requests so that the
    /// right column can display things.
    fn ensure_right_column_data(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                let selected_user = self.users_column.get_selected_entry();

                if self.accounts_column.contains_key(&selected_user.username) {
                    vec![]
                } else {
                    self.accounts_column
                        .insert(selected_user.username.clone(), Loadable::Loading);

                    vec![MessageRequest::GetAccountsOf(selected_user.clone()).into()]
                }
            }
            ColumnStackEntry::Accounts => {
                let key = self.get_account_ctx().as_key(ROOT_MAILBOX_ID);

                if self.mailbox_columns.contains_key(&key) {
                    vec![]
                } else {
                    self.mailbox_columns.insert(
                        key.clone(),
                        MailboxColumn::new(Loadable::Loading, Loadable::Loaded(vec![])),
                    );

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
                let key = self.get_account_ctx().as_key(mailbox_id);
                let middle_mailbox_column = self
                    .mailbox_columns
                    .get(&key)
                    .expect("Middle column must exist!");

                let Some(middle_column_selected_entry) = middle_mailbox_column.get_selected_entry()
                else {
                    return vec![];
                };

                match middle_column_selected_entry {
                    Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                        vec![]
                    }
                    Loadable::Loaded(entry) => match entry {
                        MailboxColumnEntry::Mailbox(mailbox_data) => {
                            let mailbox_id = mailbox_data.id.clone();
                            let key = self.get_account_ctx().as_key(Some(mailbox_id.clone()));

                            // TODO: Make sure that every mail is fetched
                            //       which can be seen
                            if self.mailbox_columns.contains_key(&key) {
                                vec![]
                            } else {
                                self.mailbox_columns.insert(
                                    key.clone(),
                                    MailboxColumn::new(Loadable::Loading, Loadable::Loading),
                                );

                                vec![
                                    MessageRequest::GetChildMailboxes {
                                        username: key.0.clone(),
                                        account_id: key.1.clone(),
                                        parent_id: key.2.clone(),
                                    }
                                    .into(),
                                    MessageRequest::QueryMails {
                                        username: key.0.clone(),
                                        account_id: key.1.clone(),
                                        mailbox: mailbox_id.clone(),
                                        window: QueryWindow {
                                            start: 0,
                                            limit: (self.terminal_height * 3) as usize,
                                        },
                                        calculate_total: true,
                                    }
                                    .into(),
                                ]
                            }
                        }
                        MailboxColumnEntry::RootMail(mail) => {
                            let key = self.get_account_ctx().as_key(mail.thread_id.clone());

                            if self.thread_columns.contains_key(&key) {
                                vec![]
                            } else {
                                self.thread_columns.insert(key.clone(), Loadable::Loading);

                                vec![
                                    MessageRequest::GetThreadMails {
                                        username: key.0,
                                        account_id: key.1,
                                        thread_id: key.2,
                                    }
                                    .into(),
                                ]
                            }
                        }
                    },
                }
            }
            ColumnStackEntry::Thread(thread_id) => {
                let thread_key = self.get_account_ctx().as_key(thread_id);
                let Some(column) = self.thread_columns.get(&thread_key).unwrap().loaded() else {
                    return vec![];
                };
                let selected_mail = column.get_selected_entry();

                let preview_key = self.get_account_ctx().as_key(selected_mail.id.clone());
                if self.mail_previews.contains_key(&preview_key) {
                    vec![]
                } else {
                    self.mail_previews
                        .insert(preview_key.clone(), Loadable::Loading);

                    vec![
                        MessageRequest::GetMailPreview {
                            username: preview_key.0,
                            account_id: preview_key.1,
                            mail_id: preview_key.2,
                        }
                        .into(),
                    ]
                }
            }
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
    Accounts,
    Mailbox(ParentMailboxId),
    Thread(ThreadId),
}

#[derive(Clone, Hash)]
struct AccountCtx {
    username: Username,
    account_id: AccountId,
}

impl AccountCtx {
    pub fn as_key<T>(&self, other: T) -> (Username, AccountId, T) {
        (self.username.clone(), self.account_id.clone(), other)
    }
}
