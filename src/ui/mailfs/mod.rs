mod message;
mod message_request;
mod types;
mod user_action;
mod view;

use crate::{
    config::{self, Username},
    datasource::types::{MutateAttribute, QueryWindow},
    types::{
        AccountData, AccountId, CachedMail, InitMailboxData, MailId, MailKeyword, MailboxId,
        ParentMailboxId, ROOT_MAILBOX_ID, ThreadId,
    },
    ui::{
        Layer, Loadable, Navigation,
        mailfs::types::{
            MailEntry, MailPreview, UserEntry,
            column::{
                AccountsColumn, Column, DEFAULT_SECTION_SIZE, MailboxColumn, MailboxColumnEntry,
                ThreadColumn, UserColumn,
            },
        },
        statusbar::{StatusMsgType, StatusbarState},
        utils::keybindmanager::{self, KeybindManager},
    },
};
use crossterm::event::Event;
use ratatui::layout::Size;
use std::{
    cell::OnceCell,
    collections::{HashMap, HashSet},
    str::FromStr,
};
use throbber_widgets_tui::ThrobberState;
use tracing::{debug, warn};
use user_action::UserAction;

pub use message::*;
pub use message_request::*;
pub use view::view;

type MailboxColumnKey = (AccountId, ParentMailboxId);
type ThreadColumnKey = (AccountId, ThreadId);
type MailPreviewKey = (AccountId, MailId);

pub struct State {
    keybindings: KeybindManager<UserAction>,
    statusbar: StatusbarState,

    throbber: ThrobberState,
    mode: Mode,

    column_stack: Vec<ColumnStackEntry>,

    users_column: UserColumn,
    accounts_column: HashMap<Username, Loadable<AccountsColumn>>,
    mailbox_columns: HashMap<MailboxColumnKey, Loadable<MailboxColumn>>,
    thread_columns: HashMap<ThreadColumnKey, Loadable<ThreadColumn>>,
    mail_previews: HashMap<MailPreviewKey, Loadable<MailPreview>>,

    column_area_size: Option<Size>,
    selection: HashMap<SelectedValue, SelectionCtx>,
}

impl State {
    pub fn new() -> (Self, UserEntry) {
        let users_column = UserColumn::new();
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
                account_id,
                mailbox_id,

                data,
            } => self.handle_init_mailbox(account_id, mailbox_id, data),
            Message::SetRootMails {
                account_id,
                mailbox,
                window,
                result,
            } => self.handle_set_root_mails(account_id, mailbox, window, result),
            Message::SetThreadMails {
                account_id,
                thread_id,
                thread_mails,
            } => self.handle_set_thread_mails(account_id, thread_id, thread_mails),
            Message::SetMailPreview {
                account_id,
                mail_id,
                preview,
            } => self.handle_set_mail_preview(account_id, mail_id, preview),
            Message::SetColumnAreaSize(size) => self.handle_set_column_area_size(size),
            Message::Reload => self.handle_reload(),
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
            UserAction::Quit => self.action_quit(),
            UserAction::OpenCommandPalette => self.action_open_command_palette(),
            UserAction::NavigateDown => self.action_navigate_down(),
            UserAction::NavigateUp => self.action_navigate_up(),
            UserAction::NavigateToTop => self.action_navigate_to_top(),
            UserAction::NavigateToBottom => self.action_navigate_to_bottom(),
            UserAction::NavigateRight => self.action_navigate_right(),
            UserAction::NavigateLeft => self.action_navigate_left(),
            UserAction::NavigateHalfPageDown => self.action_navigate_half_page_down(),
            UserAction::NavigateHalfPageUp => self.action_navigate_half_page_up(),

            UserAction::SelectEntryToggle => {
                let mut msgs = self.action_select_entry();
                msgs.extend(self.action_navigate_down());
                msgs
            }
            UserAction::CutSelectedEntries => self.action_cut_selected_entries(),
            UserAction::PasteSelectedEntries => self.action_paste_selected_entries(),

            UserAction::MoveMailboxUp => self.action_move_mailbox_up(),
            UserAction::MoveMailboxDown => self.action_move_mailbox_down(),

            UserAction::ComposeNewMail => self.action_compose_new_mail(),

            UserAction::MarkMailAsSeen => self.action_mark_mail_as_seen(),
            UserAction::MarkMailAsUnseen => self.action_mark_mail_as_unseen(),

            UserAction::ClearSelection => self.action_clear_selection(),
            UserAction::ClearStatusMessage => self.action_clear_status_msg(),

            UserAction::CreateMailbox => self.action_create_mailbox(),
            UserAction::RemoveMailbox => self.action_remove_mailbox(),
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
                let res = self
                    .accounts_column
                    .insert(username, Loadable::Loaded(AccountsColumn::new(accounts)));
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
        account_id: AccountId,
        mailbox_id: ParentMailboxId,
        data: color_eyre::Result<InitMailboxData>,
    ) -> Vec<crate::ui::Message> {
        let key = (account_id, mailbox_id.clone());

        match data {
            Ok(InitMailboxData {
                total_threads,
                child_mailboxes,
                first_mails,
            }) => {
                let is_root_mailbox = mailbox_id.is_none();
                if is_root_mailbox {
                    debug_assert!(first_mails.is_empty());
                }

                match self.mailbox_columns.get_mut(&key).unwrap() {
                    Loadable::NotLoaded => unreachable!(),
                    Loadable::Loading | Loadable::Error(_) => {
                        self.mailbox_columns.insert(
                            key,
                            Loadable::Loaded(MailboxColumn::new(
                                child_mailboxes,
                                total_threads,
                                first_mails,
                            )),
                        );
                    }
                    Loadable::Loaded(column) => {
                        column.set_entries(total_threads, child_mailboxes, first_mails);
                    }
                }
            }
            Err(err) => {
                self.mailbox_columns
                    .insert(key, Loadable::Error(err.to_string()));
            }
        };

        vec![]
    }

    fn handle_set_root_mails(
        &mut self,
        account_id: AccountId,
        mailbox_id: MailboxId,
        window: QueryWindow,
        result: color_eyre::Result<Vec<CachedMail>>,
    ) -> Vec<crate::ui::Message> {
        let key = (account_id, Some(mailbox_id.clone()));

        let column = self
            .mailbox_columns
            .get_mut(&key)
            .expect("The mailbox column itself should request this so it must be there.")
            .loaded_mut()
            .expect("Request must've come from a loaded mailbox");

        column.set_mails(window, result);
        vec![]
    }

    fn handle_set_thread_mails(
        &mut self,
        account_id: AccountId,
        thread_id: ThreadId,
        thread_mails: color_eyre::Result<Vec<CachedMail>>,
    ) -> Vec<crate::ui::Message> {
        let key = (account_id, thread_id);

        match thread_mails {
            Ok(mails) => {
                let mail_entries = mails.into_iter().map(MailEntry::from).collect();

                match self.thread_columns.get_mut(&key) {
                    Some(column) => match column.loaded_mut() {
                        Some(column) => column.set_mails(mail_entries),
                        None => *column = Loadable::Loaded(ThreadColumn::new(mail_entries)),
                    },
                    None => {
                        self.thread_columns
                            .insert(key, Loadable::Loaded(ThreadColumn::new(mail_entries)));
                    }
                }
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
        account_id: AccountId,
        mail_id: MailId,
        mail_preview: color_eyre::Result<MailPreview>,
    ) -> Vec<crate::ui::Message> {
        let key = (account_id, mail_id);

        let preview = match mail_preview {
            Ok(preview) => Loadable::Loaded(preview),
            Err(err) => Loadable::Error(err.to_string()),
        };

        self.mail_previews.insert(key, preview);

        vec![]
    }

    fn handle_set_column_area_size(&mut self, new_size: Size) -> Vec<crate::ui::Message> {
        self.column_area_size = Some(new_size);
        vec![]
    }

    fn handle_reload(&mut self) -> Vec<crate::ui::Message> {
        let username = self.get_selected_username();
        let account_id = self.get_selected_account_id();

        let mut loaded_mailboxes_keys = HashSet::new();
        let loaded_thread = OnceCell::new();
        let loaded_preview = OnceCell::new();

        let msgs = self
            .column_stack
            .iter()
            .cloned()
            .skip_while(|entry| matches!(entry, ColumnStackEntry::Users))
            .fold(Vec::new(), |mut msgs, entry| match entry {
                ColumnStackEntry::Users => unreachable!(),
                ColumnStackEntry::Accounts => msgs,
                ColumnStackEntry::Mailbox(mailbox_id) => {
                    loaded_mailboxes_keys.insert((account_id.clone(), mailbox_id.clone()));

                    let window = {
                        let section_size = self.column_area_size.map(|size| size.height as usize);
                        let key = (account_id.clone(), mailbox_id.clone());
                        let mailbox_column =
                            self.mailbox_columns.get(&key).unwrap().loaded().unwrap();
                        let range = mailbox_column.get_min_start_max_end(section_size);
                        QueryWindow {
                            start: range.start as u32,
                            limit: range.len(),
                        }
                    };

                    msgs.push(
                        MessageRequest::InitMailbox {
                            username: username.clone(),
                            account_id: account_id.clone(),
                            mailbox_id: mailbox_id.clone(),
                            mail_window: window,
                            mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                        }
                        .into(),
                    );

                    msgs
                }
                ColumnStackEntry::Thread(thread_id) => {
                    loaded_thread
                        .set((account_id.clone(), thread_id.clone()))
                        .unwrap();

                    let key = (account_id.clone(), thread_id.clone());
                    let column = self.thread_columns.get_mut(&key).unwrap().loaded().unwrap();
                    let previewed_mail = column.get_selected_entry();
                    loaded_preview
                        .set((account_id.clone(), previewed_mail.id.clone()))
                        .unwrap();

                    msgs.push(
                        MessageRequest::GetThreadMails {
                            username: username.clone(),
                            account_id: account_id.clone(),
                            thread_id,
                            mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                        }
                        .into(),
                    );

                    msgs
                }
            });

        // clear mailboxes
        self.mailbox_columns
            .retain(|mailbox_key, _column| loaded_mailboxes_keys.remove(mailbox_key));

        // thread
        match loaded_thread.get() {
            Some(key) => self
                .thread_columns
                .retain(|thread_key, _column| thread_key == key),
            None => self.thread_columns.clear(),
        }

        // preview
        match loaded_preview.get() {
            Some(key) => self
                .mail_previews
                .retain(|mail_key, _column| mail_key == key),
            None => self.mail_previews.clear(),
        }

        msgs
    }
}

/// Action implementations
impl State {
    fn action_quit(&self) -> Vec<crate::ui::Message> {
        vec![Navigation::Quit.into()]
    }

    fn action_open_command_palette(&self) -> Vec<crate::ui::Message> {
        let entries = UserAction::palette_options();
        vec![
            Navigation::OpenPalette {
                entries,
                map: |entry| Message::SelectedPaletteEntry(entry).into(),
            }
            .into(),
        ]
    }

    fn action_navigate_down(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id);
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

    fn action_navigate_up(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id);
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

    fn action_navigate_to_top(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id.clone());
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

    fn action_navigate_to_bottom(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id.clone());
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

    fn action_navigate_right(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id);
                let column = self.mailbox_columns.get(&key).unwrap().loaded().unwrap();
                let Some(selected_entry) = column.get_selected_entry() else {
                    // mailbox could be empty
                    return vec![];
                };

                match selected_entry {
                    MailboxColumnEntry::Mailbox(mailbox_data) => {
                        let key = (
                            self.get_selected_account_id(),
                            Some(mailbox_data.id.clone()),
                        );
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
                let key = (self.get_selected_account_id(), thread_id);

                match self.thread_columns.get(&key).expect("Column exists") {
                    Loadable::NotLoaded => unreachable!("Start loading?"),
                    Loadable::Loading => vec![],
                    Loadable::Error(_) => {
                        unreachable!("Middle column can't be this thread if it's an error")
                    }
                    Loadable::Loaded(column) => {
                        let selected_mail = column.get_selected_entry();
                        let mut msgs = vec![
                            Navigation::OpenReader {
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
                                mail_id: selected_mail.id.clone(),
                            }
                            .into(),
                        ];

                        if !selected_mail.keywords.contains(&MailKeyword::Seen) {
                            msgs.push(
                                MessageRequest::PatchMail {
                                    username: self.get_selected_username(),
                                    account_id: self.get_selected_account_id(),
                                    ids: vec![selected_mail.id.clone()],
                                    mailbox_ids: None,
                                    keywords: Some(MutateAttribute::Patch(vec![(
                                        MailKeyword::Seen,
                                        true,
                                    )])),
                                }
                                .into(),
                            );
                        }

                        msgs
                    }
                }
            }
        }
    }

    fn action_navigate_left(&mut self) -> Vec<crate::ui::Message> {
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

    fn action_navigate_half_page_down(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id.clone());
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

    fn action_navigate_half_page_up(&mut self) -> Vec<crate::ui::Message> {
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
                let key = (self.get_selected_account_id(), mailbox_id.clone());
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
                                username: self.get_selected_username(),
                                account_id: self.get_selected_account_id(),
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
                let key = (self.get_selected_account_id(), thread_id.clone());
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

    fn action_select_entry(&mut self) -> Vec<crate::ui::Message> {
        match self.column_stack.last().unwrap() {
            ColumnStackEntry::Users | ColumnStackEntry::Accounts => {
                self.statusbar.set_message(
                    "This action can be only applied to threads, mails and mailboxes.".to_string(),
                    StatusMsgType::Error,
                );
                return vec![];
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let selected_account_id = self.get_selected_account_id();
                let key = (selected_account_id.clone(), mailbox_id.clone());
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
                    MailboxColumnEntry::Mailbox(mailbox) => {
                        let selected = SelectedValue::Mailbox(mailbox.id.clone());
                        if self.selection.remove(&selected).is_none() {
                            self.selection.insert(selected, SelectionCtx {});
                        }
                    }
                    MailboxColumnEntry::RootMail(loadable_mail) => {
                        let Some(thread_id) =
                            loadable_mail.loaded().map(|mail| mail.thread_id.clone())
                        else {
                            return vec![];
                        };

                        let key = (selected_account_id.clone(), thread_id);
                        let Some(column) = self
                            .thread_columns
                            .get_mut(&key)
                            .expect("Thread column exists")
                            .loaded_mut()
                        else {
                            return vec![];
                        };

                        for mail in column.mails.iter_mut() {
                            let selected = SelectedValue::Mail(mail.id.clone());
                            if self.selection.remove(&selected).is_none() {
                                self.selection.insert(selected, SelectionCtx {});
                            }
                        }
                    }
                }
            }
            ColumnStackEntry::Thread(thread_id) => {
                let key = (self.get_selected_account_id(), thread_id.clone());

                let Some(column) = self
                    .thread_columns
                    .get_mut(&key)
                    .expect("Thread column exists")
                    .loaded_mut()
                else {
                    return vec![];
                };

                let mail = column.get_selected_entry_mut();

                let selected = SelectedValue::Mail(mail.id.clone());
                if self.selection.remove(&selected).is_none() {
                    self.selection.insert(selected, SelectionCtx {});
                }
            }
        };

        vec![]
    }

    fn action_cut_selected_entries(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn action_paste_selected_entries(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn action_move_mailbox_up(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn action_move_mailbox_down(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn action_compose_new_mail(&mut self) -> Vec<crate::ui::Message> {
        vec![
            Navigation::OpenComposer {
                username: self.get_selected_username(),
                account_id: self.get_selected_account_id(),
                mail_id: None,
            }
            .into(),
        ]
    }

    fn action_mark_mail_as_seen(&mut self) -> Vec<crate::ui::Message> {
        let mut msgs = Vec::with_capacity(2);
        if self.selection.is_empty() {
            msgs.extend(self.action_select_entry());
        }

        // TODO: Filter out mails which already have tags
        let ids: Vec<MailId> = self
            .selection
            .drain()
            .filter_map(|(value, _ctx)| match value {
                SelectedValue::Mailbox(_mailbox_id) => {
                    warn!("Can't mark a mailbox as 'unseen'.");
                    None
                }
                SelectedValue::Mail(mail_id) => Some(mail_id),
            })
            .collect();

        msgs.push(
            MessageRequest::PatchMail {
                username: self.get_selected_username(),
                account_id: self.get_selected_account_id(),

                ids,
                mailbox_ids: None,
                keywords: Some(MutateAttribute::Patch(vec![(MailKeyword::Seen, true)])),
            }
            .into(),
        );

        msgs
    }

    fn action_mark_mail_as_unseen(&mut self) -> Vec<crate::ui::Message> {
        let mut msgs = Vec::with_capacity(2);
        if self.selection.is_empty() {
            msgs.extend(self.action_select_entry());
        }

        // TODO: Filter out mails which already have tags
        let ids: Vec<MailId> = self
            .selection
            .drain()
            .filter_map(|(value, _ctx)| match value {
                SelectedValue::Mailbox(_mailbox_id) => {
                    warn!("Can't mark a mailbox as 'seen'.");
                    None
                }
                SelectedValue::Mail(mail_id) => Some(mail_id),
            })
            .collect();

        msgs.push(
            MessageRequest::PatchMail {
                username: self.get_selected_username(),
                account_id: self.get_selected_account_id(),
                ids,
                mailbox_ids: None,
                keywords: Some(MutateAttribute::Patch(vec![(MailKeyword::Seen, false)])),
            }
            .into(),
        );

        msgs
    }

    fn action_clear_selection(&mut self) -> Vec<crate::ui::Message> {
        self.selection.clear();
        vec![]
    }

    fn action_clear_status_msg(&mut self) -> Vec<crate::ui::Message> {
        self.statusbar.clear_message();
        vec![]
    }

    fn action_create_mailbox(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }

    fn action_remove_mailbox(&mut self) -> Vec<crate::ui::Message> {
        todo!();
    }
}

// helpers
impl State {
    fn get_selected_username(&self) -> Username {
        self.users_column.get_selected_entry().username.clone()
    }

    fn get_selected_account_id(&self) -> AccountId {
        let selected_user = self.get_selected_username();
        self.accounts_column
            .get(&selected_user)
            .unwrap()
            .loaded()
            .unwrap()
            .get_selected_entry()
            .id
            .clone()
    }

    // fn get_account_ctx(&self) -> AccountCtx {
    //     let selected_username = self.users_column.get_selected_entry().username.clone();
    //     let selected_account_id = self
    //         .accounts_column
    //         .get(&selected_username)
    //         .expect("Account column exists")
    //         .loaded()
    //         .expect("Account column is loaded")
    //         .get_selected_entry()
    //         .id
    //         .clone();

    //     AccountCtx {
    //         username: selected_username,
    //         account_id: selected_account_id,
    //     }
    // }

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
                let key = (self.get_selected_account_id(), ROOT_MAILBOX_ID);

                if self.mailbox_columns.contains_key(&key) {
                    vec![]
                } else {
                    self.mailbox_columns.insert(key.clone(), Loadable::Loading);

                    vec![
                        MessageRequest::InitMailbox {
                            username: self.get_selected_username(),
                            account_id: self.get_selected_account_id(),
                            mailbox_id: ROOT_MAILBOX_ID,
                            mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                            mail_window: QueryWindow {
                                start: 0,
                                limit: self
                                    .column_area_size
                                    .map(|size| size.height as usize)
                                    .unwrap_or(DEFAULT_SECTION_SIZE),
                            },
                        }
                        .into(),
                    ]
                }
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = (self.get_selected_account_id(), mailbox_id);
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
                        let mailbox_id = mailbox_data.id.clone();
                        let key = (self.get_selected_account_id(), Some(mailbox_id.clone()));

                        if self.mailbox_columns.contains_key(&key) {
                            vec![]
                        } else {
                            self.mailbox_columns.insert(key.clone(), Loadable::Loading);

                            vec![
                                MessageRequest::InitMailbox {
                                    username: self.get_selected_username(),
                                    account_id: self.get_selected_account_id(),
                                    mailbox_id: Some(mailbox_id),
                                    mail_properties: MailEntry::MAIL_PROPERTIES.to_vec(),
                                    mail_window: QueryWindow {
                                        start: 0,
                                        limit: self
                                            .column_area_size
                                            .map(|size| size.height as usize)
                                            .unwrap_or(DEFAULT_SECTION_SIZE),
                                    },
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
                            let thread_id = root_mail.thread_id.clone();
                            let key = (self.get_selected_account_id(), thread_id.clone());

                            if self.thread_columns.contains_key(&key) {
                                vec![]
                            } else {
                                self.thread_columns.insert(key.clone(), Loadable::Loading);

                                vec![
                                    MessageRequest::GetThreadMails {
                                        username: self.get_selected_username(),
                                        account_id: self.get_selected_account_id(),
                                        thread_id,
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
                let thread_key = (self.get_selected_account_id(), thread_id);
                let Some(column) = self.thread_columns.get(&thread_key).unwrap().loaded() else {
                    return vec![];
                };
                let selected_mail = column.get_selected_entry();

                let mail_id = selected_mail.id.clone();
                let preview_key = (self.get_selected_account_id(), mail_id.clone());
                if self.mail_previews.contains_key(&preview_key) {
                    vec![]
                } else {
                    self.mail_previews
                        .insert(preview_key.clone(), Loadable::Loading);

                    let username = self.get_selected_username();
                    let account_id = self.get_selected_account_id();

                    vec![
                        MessageRequest::GetMail {
                            username: username.clone(),
                            account_id: account_id.clone(),
                            mail_id: mail_id.clone(),
                            properties: MailPreview::MAIL_PROPERTIES.to_vec(),
                            callback: Box::new(
                                move |cached_mail: color_eyre::Result<CachedMail>| {
                                    Message::SetMailPreview {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SelectedValue {
    Mailbox(MailboxId),
    Mail(MailId),
}

#[derive(Debug)]
struct SelectionCtx {}

// enum SelectionValue {
//     Mailbox(MailboxId),
//     Mail(MailId),
//     Thread(ThreadId),
// }
