mod action;
mod mail_list;
mod mailbox_list;
mod state;

use crate::ui::{
    command_palette::{self, CommandPalette},
    mails::{mail_list::MailListWidget, mailbox_list::MailboxListWidget},
};
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use jmap_client::client::Client;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, ListState, Paragraph, StatefulWidget, Widget},
};
use state::State;
use std::{str::FromStr, sync::Arc};

#[derive(Debug)]
enum PaletteType {
    Command,
    Mailbox,
}

#[derive(Debug)]
struct PaletteCtx {
    palette: CommandPalette,
    ty: PaletteType,
}

#[derive(Debug)]
pub struct Mails {
    palette: Option<PaletteCtx>,
    state: State,

    mailbox_list_state: ListState,
    mail_list_state: ListState,
}

impl Mails {
    pub async fn new(client: Arc<Client>) -> Self {
        let state = State::new(client);

        Self {
            palette: None,
            state,

            mailbox_list_state: ListState::default(),
            mail_list_state: ListState::default(),
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        let mut actions = Vec::new();

        if let Some(command_palette) = &mut self.palette {
            if let Some(result) = command_palette.palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    command_palette::HandleEventResult::Quit => {
                        actions.push(super::Action::Quit);
                    }
                    command_palette::HandleEventResult::Selected(value) => {
                        match command_palette.ty {
                            PaletteType::Command => {
                                actions.push(Action::from_str(&value).unwrap().into())
                            }
                            PaletteType::Mailbox => {
                                todo!()
                            }
                        }
                    }
                }
            }

            return actions;
        }

        match event.code {
            KeyCode::Char('q') => actions.push(super::Action::Quit),
            KeyCode::Char(':') => actions.push(Action::OpenCommandPalette.into()),
            _ => {}
        };

        actions
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),

            Action::SelectNextMailbox => self.mailbox_list_state.select_next(),
            Action::SelectPreviousMailbox => self.mailbox_list_state.select_previous(),

            Action::SelectNextMail => self.mail_list_state.select_next(),
            Action::SelectPreviousMail => self.mail_list_state.select_previous(),

            Action::OpenMailboxPalette => {
                if let Some(mailbox_names) = self.state.get_mailbox_names() {
                    let entries = mailbox_names
                        .into_iter()
                        .map(|mailbox_name| command_palette::Entry {
                            name: mailbox_name,
                            description: "".to_string(),
                        })
                        .collect::<Vec<command_palette::Entry>>();

                    self.palette = Some(PaletteCtx {
                        palette: CommandPalette::new(entries),
                        ty: PaletteType::Mailbox,
                    });
                }
            }
            Action::OpenCommandPalette => {
                self.palette = Some(PaletteCtx {
                    palette: CommandPalette::new(Action::palette_options()),
                    ty: PaletteType::Command,
                })
            }
            Action::CloseCommandPalette => self.palette = None,
            // Action::OpenMailInPager => return Some(super::Action::OpenPager),
            // Action::CreateNewMail => return Some(super::Action::OpenComposer),
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
        self.render_mail_list(mail_list, buf);

        // self.preview.render(preview, buf);
        // self.statusbar.render(statusbar, buf);

        if let Some(command_palette) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            command_palette.palette.render(a, buf);
        }
    }
}

/// Render functions
impl Mails {
    fn render_mailbox_list(&mut self, area: Rect, buf: &mut Buffer) {
        match self.state.get_mailbox_names() {
            Some(names) => StatefulWidget::render(
                MailboxListWidget::new(&names),
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
        if let Some(selected_mailbox_idx) = self.mailbox_list_state.selected() {
            match self.state.get_mails(selected_mailbox_idx) {
                Some(names) => StatefulWidget::render(
                    MailListWidget::new(&names),
                    area,
                    buf,
                    &mut self.mail_list_state,
                ),
                None => Widget::render(
                    Paragraph::new("Loading...").block(Block::bordered()),
                    area,
                    buf,
                ),
            }
        }
        // if let Some(selected_mailbox_id) = self.state.get_selected_mailbox() {}
    }
}
