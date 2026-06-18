use crate::ui::command_palette::{Command, HandleEventResult};
use action::Action;
use crossterm::event::KeyEvent;
use jmap_client::client::Client;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Clear, Widget},
};
use std::sync::Arc;
use strum::{EnumMessage, EnumProperty, IntoEnumIterator, VariantArray};
use tokio::join;

mod action;
mod mail_list;
mod mailbox_list;
mod preview;
mod statusbar;

const INIT_FOCUS: Focus = Focus::MailList;

#[derive(Debug, PartialEq, Eq, Clone, Copy, strum::IntoStaticStr)]
#[strum(serialize_all = "Train-Case")]
enum Focus {
    MailboxList,
    MailList,
    Preview,
    CommandPalette,
}

#[derive(Debug)]
pub struct State {
    focus: Focus,

    mailbox_list: mailbox_list::State,
    mail_list: mail_list::State,
    preview: preview::State,

    statusbar: statusbar::State,
    command_palette: super::command_palette::State,
}

impl State {
    pub async fn new(client: Arc<Client>) -> Self {
        let command_palette = {
            let options = Action::iter()
                .enumerate()
                .filter_map(|(idx, variant)| {
                    if let Some(is_intern) = variant.get_bool("intern") {
                        if is_intern {
                            return None;
                        }
                    }

                    let name = variant.to_string();
                    let description = variant.get_message().unwrap().to_string();

                    Some(Command {
                        idx,
                        name,
                        description,
                    })
                })
                .collect::<Vec<Command>>();

            super::command_palette::State::new(options)
        };

        let mailbox_list_future = mailbox_list::State::new(client.clone());
        let mail_list_future = mail_list::State::new(client.clone());
        let (mailbox_list, mail_list) = join!(mailbox_list_future, mail_list_future);

        Self {
            command_palette,

            focus: INIT_FOCUS,
            mailbox_list,
            mail_list,
            preview: preview::State::new(),
            statusbar: statusbar::State::new(INIT_FOCUS.into()),
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
                            tracing::debug!("Selected: {}", idx);
                            self.command_palette.reset();
                            self.apply_action(Action::FocusMailList);
                            Action::VARIANTS[idx]
                        }
                    })
            }
        }
        .and_then(|a| self.apply_action(a))
    }

    fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),

            Action::FocusMailList => self.set_focus(Focus::MailList),
            Action::FocusMailBoxList => self.set_focus(Focus::MailboxList),
            Action::FocusPreview => self.set_focus(Focus::Preview),
            Action::FocusRightPanel => match self.focus {
                Focus::MailboxList => self.set_focus(Focus::MailList),
                Focus::MailList => self.set_focus(Focus::Preview),
                _ => {}
            },
            Action::FocusLeftPanel => match self.focus {
                Focus::MailList => self.set_focus(Focus::MailboxList),
                Focus::Preview => self.set_focus(Focus::MailList),
                _ => {}
            },

            Action::SelectNextMail => self.mail_list.select_next(),
            Action::SelectPreviousMail => self.mail_list.select_previous(),

            Action::SelectNextMailBox => self.mailbox_list.select_next(),
            Action::SelectPreviousMailBox => self.mailbox_list.select_previous(),

            Action::OpenCommandPalette => self.set_focus(Focus::CommandPalette),
            Action::OpenMailInPager => return Some(super::Action::OpenPager),

            Action::CreateNewMail => return Some(super::Action::OpenComposer),
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
            Constraint::Percentage(60),
            Constraint::Percentage(30),
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
