use crate::ui::command_palette::{Command, HandleEventResult};
use action::Action;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Clear, Widget},
};
use strum::{EnumMessage, EnumProperty, IntoEnumIterator, VariantArray};

mod action;
mod mail_list;
mod mailbox_list;
mod preview;
mod statusbar;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
enum Focus {
    MailboxList,
    #[default]
    MailList,
    Preview,
    CommandPalette,
}

#[derive(Debug, Default)]
pub struct State {
    focus: Focus,

    mailbox_list: mailbox_list::State,
    mail_list: mail_list::State,
    preview: preview::State,

    statusbar: statusbar::State,
    command_palette: super::command_palette::State,
}

impl State {
    pub fn new() -> Self {
        let options = Action::iter()
            .filter_map(|variant| {
                if variant.get_bool("intern").unwrap() {
                    return None;
                }

                let name = toml::ser::to_string(&variant).unwrap();
                let description = variant.get_message().unwrap().to_string();

                Some(Command { name, description })
            })
            .collect::<Vec<Command>>();

        Self {
            command_palette: super::command_palette::State::new(options),
            ..Default::default()
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        match self.focus {
            Focus::MailboxList => self.mailbox_list.handle_event(event),
            Focus::MailList => self.mail_list.handle_event(event),
            Focus::Preview => self.preview.handle_event(event),
            Focus::CommandPalette => {
                self.command_palette
                    .handle_event(event)
                    .map(|action| match action {
                        HandleEventResult::Quit => Action::FocusMailList,
                        HandleEventResult::Selected(idx) => {
                            self.command_palette.reset();
                            Action::VARIANTS[idx]
                        }
                    })
            }
        }
        .and_then(|a| self.apply_action(a))
    }

    fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        match a {
            Action::Quit => return Some(super::Action::Quit),

            Action::FocusMailList => self.set_focus(Focus::MailList),
            Action::FocusMailBoxList => self.set_focus(Focus::MailboxList),
            Action::FocusPreview => self.set_focus(Focus::Preview),

            Action::OpenCommandPalette => self.focus = Focus::CommandPalette,
        }

        None
    }

    fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
        self.mailbox_list.set_focus(focus == Focus::MailboxList);
        self.mail_list.set_focus(focus == Focus::MailList);
        self.preview.set_focus(focus == Focus::Preview);
    }
}

impl Widget for &mut State {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let [content, statusbar] = area.layout(&Layout::vertical([
            Constraint::Fill(0),
            Constraint::Length(3),
        ]));

        let [mail_boxes, mail_list, preview] = content.layout(&Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(50),
            Constraint::Percentage(40),
        ]));

        self.mailbox_list.render(mail_boxes, buf);
        self.mail_list.render(mail_list, buf);
        self.preview.render(preview, buf);
        self.statusbar.render(statusbar, buf);

        if self.focus == Focus::CommandPalette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            self.command_palette.render(a, buf);
        }
    }
}
