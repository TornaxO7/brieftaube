use super::Action;
use crate::ui::{ScreenPalette, ScreenState, palette, utils::keybindmanager::KeybindManager};
use chrono::DateTime;
use jmap_client::email::{Email, EmailAddress};
use ratatui::widgets::ScrollbarState;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<Action>,

    pub mail: Email,
    /// String representation of mail
    pub mail_str: String,

    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
}

impl State {
    pub fn new(mail: Email) -> Self {
        let mail_str = get_string_representation(&mail);

        let vertical = ScrollbarState::new(mail_str.lines().count());
        let horizontal =
            ScrollbarState::new(mail_str.lines().map(|line| line.len()).max().unwrap());

        Self {
            app_actions: vec![],
            palette: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::ScrollLeft),
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("l", Action::ScrollRight),
                ("q", Action::Quit),
                ("<BS>", Action::Back),
            ])),

            mail,
            mail_str,
            vertical,
            horizontal,
        }
    }

    fn scroll_down(&mut self) {
        self.vertical.next();
    }

    fn scroll_up(&mut self) {
        self.vertical.prev();
    }

    fn scroll_left(&mut self) {
        self.horizontal.prev();
    }

    fn scroll_right(&mut self) {
        self.horizontal.next();
    }
}

impl ScreenState<Action, PaletteType> for State {
    fn update(&mut self) {}

    fn apply_action(&mut self, action: Action) {
        debug!("Action: {}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(super::action::palette_options()));
            }
            Action::CloseCommandPalette => self.palette = None,

            Action::ScrollUp => self.scroll_up(),
            Action::ScrollDown => self.scroll_down(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ScrollRight => self.scroll_right(),

            Action::Back => {
                self.app_actions.push(crate::Action::Back);
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
        match result {
            palette::HandleEventResult::Cancel => {}
            palette::HandleEventResult::Selected(value) => match value {
                PaletteType::Action(action) => self.apply_action(action),
            },
        }
    }
}

fn get_string_representation(mail: &Email) -> String {
    tracing::debug!("{:#?}", mail);
    let date = DateTime::from_timestamp_millis(mail.received_at().unwrap())
        .unwrap()
        .format("%A, %d %B %Y %T");

    let from = addresses_to_string(mail.from().unwrap());
    let to = addresses_to_string(mail.to().unwrap());
    let subject = mail.subject().unwrap();

    let body = {
        let mut s = String::new();

        for body in mail.text_body().unwrap() {
            let id = body.part_id().unwrap();

            s.push_str(mail.body_value(id).expect("Body value exists").value());
        }

        s
    };

    format!(
        "\
Date: {date}
From: {from}
To: {to}
Subject: {subject}


{body}"
    )
}

fn addresses_to_string(addresses: &[EmailAddress]) -> String {
    let mut iter = addresses.iter();

    let mut s = format!("{}", iter.next().unwrap().email());

    for addr in iter {
        s.push_str(&format!(", {}", addr.email()))
    }

    s
}
