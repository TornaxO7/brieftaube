mod columns;
mod message;
mod message_request;
mod types;
mod user_action;
mod view;

use crate::{
    config::{self, Username},
    datasource::types::QueryWindow,
    types::{
        AccountData, AccountId, CachedMail, InitMailboxData, MailId, MailboxId, ParentMailboxId,
        ROOT_MAILBOX_ID, ThreadId,
    },
    ui::{
        Layer, Loadable, Navigation,
        mailfs::{
            columns::*,
            types::{MailEntry, MailPreview, UserEntry},
        },
        statusbar::StatusbarState,
        utils::keybindmanager::{self, KeybindManager},
    },
};
use crossterm::event::Event;
use ratatui::layout::Size;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::*;
pub use message_request::*;
pub use view::view;

pub struct State {
    keybindings: KeybindManager<UserAction>,
    statusbar: StatusbarState,

    throbber: ThrobberState,
    mode: Mode,

    column_stack: Vec<ColumnStackEntry>,

    // TODO: Improve abstraction
    //  - Introduce `Column` trait (`get_selected_entry`, `get_selected_entry_mut`, navigation, selection, etc.)
    users_column: columns::UserColumn,
    accounts_column: HashMap<Username, Loadable<columns::AccountsColumn>>,
    mailbox_columns:
        HashMap<(Username, AccountId, ParentMailboxId), Loadable<columns::MailboxColumn>>,
    thread_columns: HashMap<(Username, AccountId, ThreadId), Loadable<columns::ThreadColumn>>,
    mail_previews: HashMap<(Username, AccountId, MailId), Loadable<MailPreview>>,

    column_area_size: Option<Size>,
    selection: HashMap<(Username, AccountId), SelectionType>,
}

impl State {
    pub fn new() -> (Self, UserEntry) {
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
                statusbar: StatusbarState::new(),
                throbber: ThrobberState::default(),
                mode: Mode::Normal,
                column_stack: vec![ColumnStackEntry::Users],

                accounts_column,
                thread_columns,
                users_column,
                mailbox_columns,
                mail_previews,

                column_area_size: None,
                selection: HashMap::new(),

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
                    ("<C-d>", UserAction::NavigateHalfPageDown),
                    ("<C-u>", UserAction::NavigateHalfPageUp),
                ])),
            },
            initial_user,
        )
    }
}

impl Layer<Message, crate::ui::Message> for State {
    fn update(&mut self, msg: Message) -> Vec<crate::ui::Message> {
        self.throbber.calc_next();
        let mut response_msgs = match msg {
            Message::Event(event) => self.handle_event(event),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
            Message::SetUserAccounts {
                username: to,
                accounts,
            } => self.handle_set_user_accounts(to, accounts),
            Message::InitMailbox {
                username,
                account_id,
                mailbox_id,

                data,
            } => self.handle_init_mailbox(username, account_id, mailbox_id, data),
            Message::SetRootMails {
                username,
                account_id,
                mailbox,
                window,
                result,
            } => self.handle_set_root_mails(username, account_id, mailbox, window, result),
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
            Message::SetColumnAreaSize(size) => self.handle_set_column_area_size(size),
            Message::ReloadColumnEntries => self.handle_reload_column_entries(),
        };

        response_msgs.extend(self.ensure_right_column_data());

        response_msgs
            .into_iter()
            .map(crate::ui::Message::from)
            .collect()
    }
}

// Message handling
impl State {
    fn handle_event(&mut self, event: Event) -> Vec<crate::ui::Message> {
        match event {
            Event::Mouse(_) | Event::Paste(_) | Event::FocusGained | Event::FocusLost => vec![],
            Event::Resize(_, _) => {
                // TODO: If new height exceeds mails list for mailbox column => query more
                vec![]
            }
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                keybindmanager::HandleEvent::Action(action) => {
                    self.statusbar.reset_pressed_keys();
                    self.handle_user_action(action)
                }
                keybindmanager::HandleEvent::Registered => {
                    self.statusbar.register_key_event(key_event);
                    vec![]
                }
                keybindmanager::HandleEvent::Cancel => {
                    self.statusbar.reset_pressed_keys();
                    vec![]
                }
            },
        }
    }

    #[must_use]
    fn handle_user_action(&mut self, action: UserAction) -> Vec<crate::ui::Message> {
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
            UserAction::NavigateHalfPageDown => self.navigate_half_page_down(),
            UserAction::NavigateHalfPageUp => self.navigate_half_page_up(),

            UserAction::SelectEntryToggle => self.select_entry(),
            UserAction::CutSelectedEntries => self.cut_selected_entries(),
            UserAction::PasteSelectedEntries => self.paste_selected_entries(),

            UserAction::MoveMailboxUp => self.move_mailbox_up(),
            UserAction::MoveMailboxDown => self.move_mailbox_down(),

            UserAction::ComposeMarkdownMail => self.compose_markdown_mail(),

            UserAction::CreateMailbox => self.create_mailbox(),
            UserAction::RemoveMailbox => self.remove_mailbox(),
            // UserAction::MarkMailAsUnseen => self.mail_patch_keywords(&[(MailKeyword::Seen, false)]),
            // UserAction::MarkMailAsSeen => self.mail_patch_keywords(&[(MailKeyword::Seen, true)]),
        }
    }

    fn handle_selected_palette_entry(&mut self, entry: String) -> Vec<crate::ui::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        self.handle_user_action(action)
    }

    fn handle_set_user_accounts(
        &mut self,
        username: config::Username,
        accounts: color_eyre::Result<Vec<AccountData>>,
    ) -> Vec<crate::ui::Message> {
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

    fn handle_init_mailbox(
        &mut self,
        username: Username,
        account_id: AccountId,
        mailbox_id: ParentMailboxId,
        data: color_eyre::Result<InitMailboxData>,
    ) -> Vec<crate::ui::Message> {
        let key = (username, account_id, mailbox_id.clone());
        let column = self.mailbox_columns.get_mut(&key).unwrap();

        *column = match data {
            Ok(InitMailboxData {
                total_threads,
                child_mailboxes,
                first_mails,
            }) => {
                let is_root_mailbox = mailbox_id.is_none();
                if is_root_mailbox {
                    debug_assert!(first_mails.is_empty());
                }

                let first_mails: Vec<MailEntry> =
                    first_mails.into_iter().map(MailEntry::from).collect();

                Loadable::Loaded(MailboxColumn::new(
                    child_mailboxes,
                    total_threads,
                    first_mails,
                ))
            }
            Err(err) => Loadable::Error(err.to_string()),
        };

        vec![]
    }

    fn handle_set_root_mails(
        &mut self,
        username: Username,
        account_id: AccountId,
        mailbox_id: MailboxId,
        window: QueryWindow,
        result: color_eyre::Result<Vec<CachedMail>>,
    ) -> Vec<crate::ui::Message> {
        let key = (username, account_id, Some(mailbox_id.clone()));

        let column = self
            .mailbox_columns
            .get_mut(&key)
            .expect("The mailbox column itself should request this so it must be there.")
            .loaded_mut()
            .expect("Request must've come from a loaded mailbox");

        column.set_mailbox_mails(window, result);
        vec![]
    }

    fn handle_set_thread_mails(
        &mut self,
        username: Username,
        account_id: AccountId,
        thread_id: ThreadId,
        thread_mails: color_eyre::Result<Vec<CachedMail>>,
    ) -> Vec<crate::ui::Message> {
        let key = (username, account_id, thread_id);

        match thread_mails {
            Ok(mails) => {
                let mail_entries = mails.into_iter().map(MailEntry::from).collect();

                let res = self
                    .thread_columns
                    .insert(key, Loadable::Loaded(ThreadColumn::new(mail_entries)));

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
        mail_preview: color_eyre::Result<MailPreview>,
    ) -> Vec<crate::ui::Message> {
        let key = (username, account_id, mail_id);

        let current_preview = self.mail_previews.get_mut(&key).unwrap();

        *current_preview = match mail_preview {
            Ok(preview) => Loadable::Loaded(preview),
            Err(err) => Loadable::Error(err.to_string()),
        };

        vec![]
    }

    fn handle_set_column_area_size(&mut self, new_size: Size) -> Vec<crate::ui::Message> {
        self.column_area_size = Some(new_size);
        vec![]
    }

    fn handle_reload_column_entries(&mut self) -> Vec<crate::ui::Message> {
        // TODO: HERE
        todo!("Some entries changed => Unload every column")
    }
}

/// Action implementations
impl State {
    fn quit(&self) -> Vec<crate::ui::Message> {
        vec![Navigation::Quit.into()]
    }

    fn open_command_palette(&self) -> Vec<crate::ui::Message> {
        let entries = UserAction::palette_options();
        vec![
            Navigation::OpenPalette {
                entries,
                map: |entry| Message::SelectedPaletteEntry(entry).into(),
            }
            .into(),
        ]
    }

    fn navigate_down(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_down(1);
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

                column.navigate_down(1);
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                let Some(column) = self
                    .mailbox_columns
                    .get_mut(&key)
                    .expect("Mailbox column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };
                column.navigate_down(1);

                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }

                msgs.extend(self.ensure_right_column_data());
                msgs
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

                column.navigate_down(1);
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_up(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap().clone() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_up(1);
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

                column.navigate_up(1);
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                let Some(column) = self.mailbox_columns.get_mut(&key).unwrap().loaded_mut() else {
                    return vec![];
                };
                column.navigate_up(1);

                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }
                msgs.extend(self.ensure_right_column_data());
                msgs
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

                column.navigate_up(1);
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_to_top(&mut self) -> Vec<crate::ui::Message> {
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
                let Some(column) = self.mailbox_columns.get_mut(&key).unwrap().loaded_mut() else {
                    return vec![];
                };

                column.navigate_to_top();
                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }

                msgs.extend(self.ensure_right_column_data());
                msgs
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

    fn navigate_to_bottom(&mut self) -> Vec<crate::ui::Message> {
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
                let Some(column) = self.mailbox_columns.get_mut(&key).unwrap().loaded_mut() else {
                    return vec![];
                };
                column.navigate_to_bottom();
                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }

                msgs.extend(self.ensure_right_column_data());
                msgs
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

    fn navigate_right(&mut self) -> Vec<crate::ui::Message> {
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
                let column = self.mailbox_columns.get(&key).unwrap().loaded().unwrap();
                let Some(selected_entry) = column.get_selected_entry() else {
                    // mailbox could be empty
                    return vec![];
                };

                match selected_entry {
                    MailboxColumnEntry::Mailbox(mailbox_data) => {
                        let key = self.get_account_ctx().as_key(Some(mailbox_data.id.clone()));
                        let next_mailbox_column_is_loaded =
                            self.mailbox_columns.get(&key).unwrap().loaded().is_some();

                        if next_mailbox_column_is_loaded {
                            self.column_stack
                                .push(ColumnStackEntry::Mailbox(Some(mailbox_data.id.clone())));
                            self.ensure_right_column_data()
                        } else {
                            vec![]
                        }
                    }
                    MailboxColumnEntry::RootMail(loadable_root_mail) => {
                        if let Some(root_mail) = loadable_root_mail.loaded() {
                            self.column_stack
                                .push(ColumnStackEntry::Thread(root_mail.thread_id.clone()));
                            self.ensure_right_column_data()
                        } else {
                            vec![]
                        }
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
                        vec![
                            Navigation::OpenReader {
                                username: key.0,
                                account_id: key.1,
                                mail_id: selected_mail.id.clone(),
                            }
                            .into(),
                        ]
                    }
                }
            }
        }
    }

    fn navigate_left(&mut self) -> Vec<crate::ui::Message> {
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

    fn navigate_half_page_down(&mut self) -> Vec<crate::ui::Message> {
        let Some(size) = self.column_area_size else {
            return vec![];
        };

        let offset = size.height / 2;

        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_down(offset);
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

                column.navigate_down(offset);
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                let Some(column) = self
                    .mailbox_columns
                    .get_mut(&key)
                    .expect("Column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_down(offset);
                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }

                msgs.extend(self.ensure_right_column_data());
                msgs
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

                column.navigate_down(offset);
                self.ensure_right_column_data()
            }
        }
    }

    fn navigate_half_page_up(&mut self) -> Vec<crate::ui::Message> {
        let Some(size) = self.column_area_size else {
            return vec![];
        };

        let offset = size.height / 2;

        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                self.users_column.navigate_up(offset);
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

                column.navigate_up(offset);
                self.ensure_right_column_data()
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id.clone());
                let Some(column) = self
                    .mailbox_columns
                    .get_mut(&key)
                    .expect("Column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                column.navigate_up(offset);
                let mut msgs = Vec::new();
                if let Some(mailbox_id) = mailbox_id {
                    let section_size = self.column_area_size.map(|size| size.height as usize);
                    msgs.extend(column.ensure_loaded_mails(section_size).into_iter().map(
                        |window| {
                            MessageRequest::QueryMails {
                                username: key.0.clone(),
                                account_id: key.1.clone(),
                                mailbox: mailbox_id.clone(),
                                mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                window,
                            }
                            .into()
                        },
                    ));
                }

                msgs.extend(self.ensure_right_column_data());
                msgs
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

                column.navigate_up(offset);
                self.ensure_right_column_data()
            }
        }
    }

    fn select_entry(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users => {
                let entry = self.users_column.get_selected_entry_mut();
                entry.is_selected = !entry.is_selected;
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

                let entry = column.get_selected_entry_mut();
                entry.is_selected = !entry.is_selected;
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let account_ctx = self.get_account_ctx();
                let key = account_ctx.as_key(mailbox_id.clone());
                let Some(column) = self
                    .mailbox_columns
                    .get_mut(&key)
                    .expect("Mailbox column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                let Some(entry) = column.get_selected_entry_mut() else {
                    return vec![];
                };

                match entry {
                    MailboxColumnEntryMut::Mailbox(mailbox_entry) => {
                        mailbox_entry.is_selected = !mailbox_entry.is_selected;
                    }
                    MailboxColumnEntryMut::RootMail(loadable_mail) => {
                        let Some(mail_entry) = loadable_mail.loaded_mut() else {
                            return vec![];
                        };

                        mail_entry.is_selected = !mail_entry.is_selected;

                        // also select every mail in the thread
                        let key = account_ctx.as_key(mail_entry.thread_id.clone());
                        let Some(column) = self
                            .thread_columns
                            .get_mut(&key)
                            .expect("Thread column exists")
                            .loaded_mut()
                        else {
                            return vec![];
                        };

                        for mail in column.mails.iter_mut() {
                            mail.is_selected = !mail.is_selected;
                        }
                    }
                }
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = self.get_account_ctx().as_key(thread_id.clone());

                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Thread column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                let selected_mail = column.get_selected_entry_mut();
                selected_mail.is_selected = !selected_mail.is_selected;
            }
        };

        self.navigate_down()
    }

    fn cut_selected_entries(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn paste_selected_entries(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn move_mailbox_up(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn move_mailbox_down(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn compose_markdown_mail(&mut self) -> Vec<crate::ui::Message> {
        let account_ctx = self.get_account_ctx();

        vec![
            Navigation::OpenComposer {
                username: account_ctx.username,
                account_id: account_ctx.account_id,
                mail_id: None,
            }
            .into(),
        ]
    }

    fn create_mailbox(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn remove_mailbox(&mut self) -> Vec<crate::ui::Message> {
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
                    self.mailbox_columns.insert(key.clone(), Loadable::Loading);

                    vec![
                        MessageRequest::InitMailbox {
                            username: key.0,
                            account_id: key.1,
                            mailbox_id: key.2,
                            mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                            max_init_mails: self
                                .column_area_size
                                .map(|size| size.height as usize)
                                .unwrap_or(columns::DEFAULT_SECTION_SIZE),
                        }
                        .into(),
                    ]
                }
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = self.get_account_ctx().as_key(mailbox_id);
                let Some(middle_mailbox_column) = self
                    .mailbox_columns
                    .get(&key)
                    .expect("Middle column must exist!")
                    .loaded()
                else {
                    return vec![];
                };

                let Some(middle_column_selected_entry) = middle_mailbox_column.get_selected_entry()
                else {
                    return vec![];
                };

                match middle_column_selected_entry {
                    MailboxColumnEntry::Mailbox(mailbox_data) => {
                        let key = self.get_account_ctx().as_key(Some(mailbox_data.id.clone()));

                        if self.mailbox_columns.contains_key(&key) {
                            vec![]
                        } else {
                            self.mailbox_columns.insert(key.clone(), Loadable::Loading);

                            vec![
                                MessageRequest::InitMailbox {
                                    username: key.0,
                                    account_id: key.1,
                                    mailbox_id: key.2,
                                    mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                    max_init_mails: self
                                        .column_area_size
                                        .map(|size| size.height as usize)
                                        .unwrap_or(columns::DEFAULT_SECTION_SIZE),
                                }
                                .into(),
                            ]
                        }
                    }
                    MailboxColumnEntry::RootMail(loadable_root_mail) => match loadable_root_mail {
                        Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                            vec![]
                        }
                        Loadable::Loaded(root_mail) => {
                            let key = self.get_account_ctx().as_key(root_mail.thread_id.clone());

                            if self.thread_columns.contains_key(&key) {
                                vec![]
                            } else {
                                self.thread_columns.insert(key.clone(), Loadable::Loading);

                                vec![
                                    MessageRequest::GetThreadMails {
                                        username: key.0,
                                        account_id: key.1,
                                        thread_id: key.2,
                                        mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
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

                    let username = preview_key.0;
                    let account_id = preview_key.1;
                    let mail_id = preview_key.2;

                    vec![
                        MessageRequest::GetMail {
                            username: username.clone(),
                            account_id: account_id.clone(),
                            mail_id: mail_id.clone(),
                            properties: MailPreview::MAIL_PROPERTIES.to_vec(),
                            callback: Box::new(
                                move |cached_mail: color_eyre::Result<CachedMail>| {
                                    Message::SetMailPreview {
                                        username: username.clone(),
                                        account_id: account_id.clone(),
                                        mail_id: mail_id.clone(),
                                        preview: cached_mail.map(MailPreview::from),
                                    }
                                    .into()
                                },
                            ),
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

enum SelectionType {
    User(Username),
    Mailbox(MailboxId),
    Mail(MailId),
    Thread(ThreadId),
}
