mod user_action;
mod view;

use crate::{
    types::{MailKeyword, ParentMailboxId},
    ui::{
        Action, LayerCore, LayerMessage, LayerState,
        utils::keybindmanager::{self, KeybindManager},
    },
};
use crossterm::event::Event;
use ratatui::widgets::ListState;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use view::view;

enum OverlayValue {
    Action,
    NewMailboxName,
}

pub struct State {
    keybindings: KeybindManager<UserAction>,
    overlay_value: Option<OverlayValue>,

    pub throbber: ThrobberState,
    pub account_column: ListState,
    pub navigation_stack: Vec<ParentMailboxId>,
    pub mailboxes: HashMap<ParentMailboxId, ListState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            overlay_value: None,
            throbber: ThrobberState::default(),
            account_column: ListState::default(),
            navigation_stack: vec![],
            mailboxes: HashMap::new(),
            // selection: HashMap::new(),
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

impl From<State> for Option<LayerMessage> {
    fn from(_: State) -> Self {
        None
    }
}

impl LayerCore for State {
    fn handle_event(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Mouse(_)
            | Event::Paste(_)
            | Event::Resize(_, _)
            | Event::FocusGained
            | Event::FocusLost => None,
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                keybindmanager::HandleEvent::Action(action) => self.apply_action(action),
                keybindmanager::HandleEvent::Registered => None,
                keybindmanager::HandleEvent::Cancel => None,
            },
        }
    }

    fn handle_layer_message<Msg>(&mut self, msg: Msg) -> Option<Action>
    where
        Msg: Into<Option<LayerMessage>>,
    {
        let expected_type = self.overlay_value.take()?;
        let msg = msg.into()?;

        match expected_type {
            OverlayValue::Action => {
                let action = UserAction::from_str(msg.as_str()).unwrap();
                self.apply_action(action)
            }
            OverlayValue::NewMailboxName => {
                // let column_id = self.center_column_mailbox().clone();
                // let columns = self.columns.clone();
                // let backend = self.backend.clone();
                // self.task_manager.spawn(async move {
                //     create_new_mailbox(msg, column_id, columns, backend).await;
                // });

                // None
                todo!()
            }
        }
    }
}

impl LayerState<UserAction> for State {
    fn apply_action(&mut self, action: UserAction) -> Option<Action> {
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
}

/// Action implementations
impl State {
    fn quit(&self) -> Option<Action> {
        Some(Action::Quit)
    }

    fn open_command_palette(&mut self) -> Option<Action> {
        self.overlay_value = Some(OverlayValue::Action);
        let entries = UserAction::palette_options();
        Some(Action::OpenPalette { entries })
    }

    fn navigate_down(&self) -> Option<Action> {
        todo!();
        None
    }

    fn navigate_up(&self) -> Option<Action> {
        todo!();
        None
    }

    fn navigate_to_top(&mut self) -> Option<Action> {
        todo!();
        None
    }

    fn navigate_to_bottom(&mut self) -> Option<Action> {
        todo!();
        None
    }

    fn navigate_right(&mut self) -> Option<Action> {
        todo!();
    }

    fn navigate_left(&mut self) -> Option<Action> {
        todo!();
    }

    fn navigate_to_parent(&mut self) -> Option<Action> {
        todo!();
    }

    fn select_entry(&mut self) -> Option<Action> {
        todo!();
    }

    fn cut_selected_entries(&mut self) -> Option<Action> {
        todo!();
    }

    fn paste_selected_entries(&mut self) -> Option<Action> {
        todo!();
    }

    fn move_mailbox_up(&mut self) -> Option<Action> {
        todo!();
    }

    fn move_mailbox_down(&mut self) -> Option<Action> {
        todo!();
    }

    fn create_mailbox(&mut self) -> Option<Action> {
        self.overlay_value = Some(OverlayValue::NewMailboxName);
        Some(Action::OpenPrompt {
            description: "Mailbox name:".to_string(),
        })
    }

    fn remove_mailbox(&mut self) -> Option<Action> {
        todo!();
    }

    fn mail_patch_keywords(&mut self, patch: &[(MailKeyword, bool)]) -> Option<Action> {
        todo!();
    }
}
