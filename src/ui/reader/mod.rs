mod message;
mod user_action;
mod view;

use crate::ui::{
    Layer,
    utils::keybindmanager::{HandleEvent, KeybindManager},
};
use crossterm::event::Event;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::Message;
pub use view::view;

pub struct State {
    keybindings: KeybindManager<UserAction>,
    throbber: ThrobberState,
}

impl State {
    pub fn new() -> Self {
        Self {
            throbber: ThrobberState::default(),
            keybindings: KeybindManager::new(HashMap::from([("q", UserAction::Quit)])),
        }
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        self.throbber.calc_next();

        match msg {
            Message::UserAction(action) => self.handle_user_action(action),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
            Message::Event(event) => self.handle_event(event),
        }
    }
}

// message handlers
impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::FocusGained
            | Event::FocusLost
            | Event::Mouse(_)
            | Event::Paste(_)
            | Event::Resize(_, _) => vec![],
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                HandleEvent::Action(action) => self.handle_user_action(action),
                HandleEvent::Registered => vec![],
                HandleEvent::Cancel => vec![],
            },
        }
    }

    fn handle_user_action(&mut self, action: UserAction) -> Vec<super::Message> {
        debug!("{:?}", action);

        match action {
            UserAction::OpenCommandPalette => self.open_command_palette(),
            UserAction::NavigateDown => todo!(),
            UserAction::NavigateUp => todo!(),
            UserAction::NavigateToTop => todo!(),
            UserAction::NavigateToBottom => todo!(),
            UserAction::NavigateRight => todo!(),
            UserAction::NavigateLeft => todo!(),
            UserAction::Quit => todo!(),
            UserAction::Back => self.back(),
        }
    }

    fn handle_selected_palette_entry(&mut self, entry: String) -> Vec<super::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        vec![super::Message::Reader(Message::UserAction(action))]
    }
}

// user-action handlers
impl State {
    fn open_command_palette(&self) -> Vec<super::Message> {
        let entries = UserAction::palette_options();

        vec![super::Message::OpenPalette {
            entries,
            map: |entry| super::Message::Reader(Message::SelectedPaletteEntry(entry)),
        }]
    }

    fn back(&self) -> Vec<super::Message> {
        vec![super::Message::Back]
    }
}
