use std::str::FromStr;

use crate::ui::command_palette::{Command, HandleEventResult};
use action::Action;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Clear, Widget},
};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator};

mod action;
mod attachment;
mod mail;

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "Train-Case")]
enum Focus {
    CommandPalette,
    Mail,
    Attachments,
}

#[derive(Debug)]
pub struct State {
    focus: Focus,

    mail: mail::State,
    attachments: attachment::State,
    command_palette: super::command_palette::CommandPalette,
}

impl State {
    pub fn new() -> Self {
        let options = Action::iter()
            .filter_map(|variant| {
                if let Some(is_intern) = variant.get_bool("intern") {
                    if is_intern {
                        return None;
                    }
                }

                let name = variant.to_string();
                let description = variant.get_message().unwrap().to_string();

                Some(Command { name, description })
            })
            .collect::<Vec<Command>>();

        Self {
            focus: Focus::Mail,

            mail: mail::State::new(),
            attachments: attachment::State::new(),
            command_palette: super::command_palette::CommandPalette::new(options),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match self.focus {
            Focus::Mail => self.mail.handle_event(event),
            Focus::Attachments => self.attachments.handle_event(event),
            Focus::CommandPalette => {
                self.command_palette
                    .handle_event(event)
                    .map(|action| match action {
                        HandleEventResult::Quit => Action::FocusMailPanel,
                        HandleEventResult::Selected(cmd_name) => {
                            self.command_palette.reset();
                            self.apply_action(Action::FocusMailPanel);
                            Action::from_str(cmd_name.as_str()).unwrap()
                        }
                    })
            }
        }
        .and_then(|a| self.apply_action(a))
    }

    fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => self.set_focus(Focus::CommandPalette),

            Action::FocusMailPanel => self.set_focus(Focus::Mail),
            Action::FocusAttachmentsPanel => self.set_focus(Focus::Attachments),
            Action::OpenMailList => return Some(super::Action::OpenMailList),
        }

        None
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
        self.mail.set_focus(focus == Focus::Mail);
        self.attachments.set_focus(focus == Focus::Attachments);
    }
}

impl Widget for &mut State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [content, attachments] = area.layout(&Layout::vertical([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ]));

        self.mail.render(content, buf);
        self.attachments.render(attachments, buf);

        if self.focus == Focus::CommandPalette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            self.command_palette.render(a, buf);
        }
    }
}
