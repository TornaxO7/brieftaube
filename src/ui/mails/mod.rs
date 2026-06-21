mod action;
mod mail_list;
mod mailbox_list;
mod state;

use crate::ui::{
    command_palette::{Command, HandleEventResult},
    mails::mailbox_list::MailboxListWidget,
};
use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use jmap_client::client::Client;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, ListState, Paragraph, StatefulWidget, Widget},
};
use state::State;
use std::sync::Arc;
use strum::{EnumMessage, EnumProperty, IntoEnumIterator, VariantArray};

#[derive(Debug)]
pub struct Mails {
    open_command_palette: bool,
    command_palette: super::command_palette::State,
    state: State,

    mailbox_list_state: ListState,
    mail_list_state: ListState,
}

impl Mails {
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

        let state = State::new(client);

        Self {
            open_command_palette: false,
            command_palette,
            state,

            mailbox_list_state: ListState::default(),
            mail_list_state: ListState::default(),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Option<super::Action> {
        if self.open_command_palette {
            return self.command_palette.handle_event(event).and_then(|result| {
                self.command_palette.reset();
                self.apply_action(Action::CloseCommandPalette);

                match result {
                    HandleEventResult::Quit => None,
                    HandleEventResult::Selected(idx) => self.apply_action(Action::VARIANTS[idx]),
                }
            });
        }

        match event.code {
            KeyCode::Char('q') => Some(super::Action::Quit),
            KeyCode::Char(':') => self.apply_action(Action::OpenCommandPalette),
            _ => None,
        }
    }

    fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),

            Action::SelectNextMailbox => self.mailbox_list_state.select_next(),
            Action::SelectPreviousMailbox => self.mailbox_list_state.select_previous(),

            Action::SelectNextMail => self.mail_list_state.select_next(),
            Action::SelectPreviousMail => self.mail_list_state.select_previous(),

            Action::OpenCommandPalette => self.open_command_palette = true,
            Action::CloseCommandPalette => self.open_command_palette = false,

            Action::OpenMailInPager => return Some(super::Action::OpenPager),

            Action::CreateNewMail => return Some(super::Action::OpenComposer),
        }

        None
    }
}

impl Widget for &mut Mails {
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

        self.render_mailbox_list(mail_boxes, buf);

        // self.mailbox_list.render(mail_boxes, buf);
        // self.mail_list.render(mail_list, buf);
        // self.preview.render(preview, buf);
        // self.statusbar.render(statusbar, buf);

        if self.open_command_palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            self.command_palette.render(a, buf);
        }
    }
}

/// Render functions
impl Mails {
    fn render_mailbox_list(&mut self, area: Rect, buf: &mut Buffer) {
        match self.state.get_mailbox_names() {
            Some(names) => StatefulWidget::render(
                MailboxListWidget::new(names),
                area,
                buf,
                &mut self.mailbox_list_state,
            ),
            None => Widget::render(
                Paragraph::new("Loading...").block(Block::bordered()),
                area,
                buf,
            ),
        }
    }

    fn render_mail_list(&mut self, area: Rect, buf: &mut Buffer) {
        todo!()
        // if let Some(selected_mailbox_id) = self.state.get_selected_mailbox() {}
    }
}
