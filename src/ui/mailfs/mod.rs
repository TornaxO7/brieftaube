mod user_action;
mod view;

use crate::{
    CONFIG,
    config::UserConfig,
    task_manager::TaskManager,
    types::MailKeyword,
    ui::{
        Layer,
        utils::{
            Loadable,
            keybindmanager::{self, KeybindManager},
        },
    },
};
use crossterm::event::Event;
use ratatui::widgets::ListState;
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

struct AccountCtx {}

pub struct State {
    keybindings: KeybindManager<UserAction>,
    task_manager: Rc<TaskManager>,

    pub throbber: ThrobberState,

    pub user_list: Vec<UserCtx>,
    pub user_list_state: ListState,
}

impl State {
    pub fn new(task_manager: Rc<TaskManager>) -> Self {
        let user_list: Vec<UserCtx> = CONFIG
            .get()
            .unwrap()
            .users
            .iter()
            .map(UserCtx::new)
            .collect();

        let user_list_state = if user_list.is_empty() {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(0))
        };

        Self {
            throbber: ThrobberState::default(),
            task_manager,
            // navigation_stack: vec![],
            // mailbox_states: HashMap::new(),
            // selection: HashMap::new(),
            user_list,
            user_list_state,
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
    fn update(&mut self, msg: Message) -> Option<super::Message> {
        match msg {
            Message::Event(event) => self.handle_event(event),
            Message::UserAction(action) => self.handle_user_action(action),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
        }
    }
}

impl State {
    fn handle_event(&mut self, event: Event) -> Option<super::Message> {
        match event {
            Event::Mouse(_)
            | Event::Paste(_)
            | Event::Resize(_, _)
            | Event::FocusGained
            | Event::FocusLost => None,
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                keybindmanager::HandleEvent::Action(action) => self.handle_user_action(action),
                keybindmanager::HandleEvent::Registered => None,
                keybindmanager::HandleEvent::Cancel => None,
            },
        }
    }

    fn handle_user_action(&mut self, action: UserAction) -> Option<super::Message> {
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

    fn handle_selected_palette_entry(&mut self, entry: String) -> Option<super::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        Some(super::Message::Mailfs(Message::UserAction(action)))
    }
}

/// Action implementations
impl State {
    fn quit(&self) -> Option<super::Message> {
        Some(super::Message::Quit)
    }

    fn open_command_palette(&mut self) -> Option<super::Message> {
        let entries = UserAction::palette_options();
        Some(super::Message::OpenPalette { entries })
    }

    fn navigate_down(&self) -> Option<super::Message> {
        todo!();
        None
    }

    fn navigate_up(&self) -> Option<super::Message> {
        todo!();
        None
    }

    fn navigate_to_top(&mut self) -> Option<super::Message> {
        todo!();
        None
    }

    fn navigate_to_bottom(&mut self) -> Option<super::Message> {
        todo!();
        None
    }

    fn navigate_right(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn navigate_left(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn navigate_to_parent(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn select_entry(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn cut_selected_entries(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn paste_selected_entries(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn move_mailbox_up(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn move_mailbox_down(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn create_mailbox(&mut self) -> Option<super::Message> {
        self.overlay_value = Some(OverlayValue::NewMailboxName);
        Some(super::Message::OpenPrompt {
            description: "Mailbox name:".to_string(),
        })
    }

    fn remove_mailbox(&mut self) -> Option<super::Message> {
        todo!();
    }

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) -> Option<super::Message> {
        todo!();
    }
}

pub struct UserCtx {
    pub collapsed: bool,
    pub accounts: Loadable<Vec<()>>,
    config: UserConfig,
}

impl UserCtx {
    pub fn new(config: &UserConfig) -> Self {
        Self {
            collapsed: true,
            accounts: Loadable::NotLoaded,
            config: config.clone(),
        }
    }
}
