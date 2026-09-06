mod user_action;
mod view;

use crate::{
    CONFIG,
    config::UserConfig,
    task_manager::TaskManager,
    types::{AccountData, MailKeyword, ParentMailboxId},
    ui::{
        Layer,
        utils::{
            Loadable,
            keybindmanager::{self, KeybindManager},
        },
    },
};
use crossterm::event::Event;
use ratatui::widgets::{ListState, TableState};
use std::{collections::HashMap, rc::Rc, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use view::view;

pub enum Message {
    Event(Event),
    UserAction(UserAction),

    SelectedPaletteEntry(String),
}

pub struct State {
    keybindings: KeybindManager<UserAction>,
    task_manager: Rc<TaskManager>,

    throbber: ThrobberState,
    mode: Mode,

    navigation_stack: Vec<ParentMailboxId>,
    user_and_accounts_list: Vec<UserAccountEntry>,
    user_and_accounts_list_state: TableState,
}

impl State {
    pub fn new(task_manager: Rc<TaskManager>) -> Self {
        let user_list: Vec<UserAccountEntry> = CONFIG
            .get()
            .unwrap()
            .users
            .iter()
            .cloned()
            .map(UserAccountEntry::User)
            .collect();

        let user_list_state = if user_list.is_empty() {
            TableState::default()
        } else {
            TableState::default().with_selected(Some(0))
        };

        Self {
            throbber: ThrobberState::default(),
            task_manager,
            mode: Mode::Normal,
            navigation_stack: vec![],
            // mailbox_states: HashMap::new(),
            // selection: HashMap::new(),
            user_and_accounts_list: user_list,
            user_and_accounts_list_state: user_list_state,
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
}

/// Action implementations
impl State {
    fn quit(&self) -> Vec<super::Message> {
        vec![super::Message::Quit]
    }

    fn open_command_palette(&mut self) -> Vec<super::Message> {
        let entries = UserAction::palette_options();
        vec![super::Message::OpenPalette {
            entries,
            map: |entry| super::Message::Mailfs(Message::SelectedPaletteEntry(entry)),
        }]
    }

    fn navigate_down(&self) -> Vec<super::Message> {
        todo!();
    }

    fn navigate_up(&self) -> Vec<super::Message> {
        todo!();
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn navigate_right(&mut self) -> Vec<super::Message> {
        todo!();
    }

    fn navigate_left(&mut self) -> Vec<super::Message> {
        todo!();
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

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) -> Vec<super::Message> {
        todo!();
    }
}

enum UserAccountEntry {
    User(UserConfig),
    Account(AccountData),
}

#[derive(strum::Display, Debug, Clone, Copy)]
enum Mode {
    Normal,
}
