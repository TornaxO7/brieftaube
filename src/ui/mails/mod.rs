mod action;
mod mail_list;
mod mailbox_list;
mod state;

use crate::{
    backend,
    ui::{
        command_palette::{self, CommandPalette},
        mails::{mail_list::MailListWidget, mailbox_list::MailboxListWidget},
    },
};
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use state::State;
use std::{str::FromStr, sync::Arc};

const MAILBOX_PANEL_TITLE: &str = "Mailboxes";
const MAIL_LIST_PANEL_TITLE: &str = "Mails";
const PREVIEW_PANEL_TITLE: &str = "Mail content";

#[derive(Debug)]
enum PaletteType {
    /// Palette is displaying commands
    Command,

    /// Palette is displaying mailboxes
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
}

impl Mails {
    pub async fn new(account: Arc<backend::Account>) -> Self {
        let state = State::new(account);

        Self {
            palette: None,
            state,
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        let mut actions = Vec::new();

        if let Some(command_palette) = &mut self.palette {
            if let Some(result) = command_palette.palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    command_palette::HandleEventResult::Cancel => {
                        actions.push(Action::CloseCommandPalette.into());
                    }
                    command_palette::HandleEventResult::Selected(value) => {
                        match command_palette.ty {
                            PaletteType::Command => {
                                actions.push(Action::from_str(&value).unwrap().into())
                            }
                            PaletteType::Mailbox => {
                                actions.push(Action::SelectMailbox(value).into());
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
            KeyCode::Char('j') => actions.push(Action::SelectNextMail.into()),
            KeyCode::Char('k') => actions.push(Action::SelectPreviousMail.into()),
            _ => {}
        };

        actions
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        tracing::debug!("Action: {:?}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),

            Action::SelectNextMailbox => self.state.select_next_mailbox(),
            Action::SelectPreviousMailbox => self.state.select_previous_mailbox(),
            Action::SelectMailbox(selected_name) => {
                let mailbox_names = self.state.get_mailbox_names().unwrap();

                for (idx, name) in mailbox_names.into_iter().enumerate() {
                    if name == selected_name {
                        self.state.select_mailbox(idx);
                        return None;
                    }
                }

                unreachable!("Man... why does this happen? ._.");
            }

            Action::SelectNextMail => self.state.select_next_mail(),
            Action::SelectPreviousMail => self.state.select_previous_mail(),

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
        let [content, _statusbar] = area.layout(&Layout::vertical([
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
        self.render_preview(preview, buf);
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
                MailboxListWidget::new(&names).block(Block::bordered().title(MAILBOX_PANEL_TITLE)),
                area,
                buf,
                self.state.get_mailbox_list_state_mut(),
            ),
            None => Widget::render(
                Paragraph::new("Loading...").block(Block::bordered().title(MAILBOX_PANEL_TITLE)),
                area,
                buf,
            ),
        }
    }

    fn render_mail_list(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(selected_mailbox_idx) = self.state.selected_mailbox_idx() {
            match self.state.get_mails(selected_mailbox_idx) {
                Some(names) => StatefulWidget::render(
                    MailListWidget::new(&names)
                        .block(Block::bordered().title(MAIL_LIST_PANEL_TITLE)),
                    area,
                    buf,
                    self.state.get_mail_list_state_mut(),
                ),
                None => Widget::render(
                    Paragraph::new("Loading...")
                        .block(Block::bordered().title(MAIL_LIST_PANEL_TITLE)),
                    area,
                    buf,
                ),
            }
        } else {
            Widget::render(
                Paragraph::new("No mailbox selected")
                    .block(Block::bordered().title(MAIL_LIST_PANEL_TITLE)),
                area,
                buf,
            )
        }
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(selected_mailbox_idx) = self.state.selected_mailbox_idx() {
            if let Some(selected_mail_idx) = self.state.selected_mail_list_idx() {
                if let Some(mail) = self.state.get_mail(selected_mailbox_idx, selected_mail_idx) {
                    Widget::render(
                        Paragraph::new(mail.preview().unwrap())
                            .block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
                        area,
                        buf,
                    );
                    return;
                }
            }
        }

        Widget::render(
            Paragraph::new("No mail selected").block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
            area,
            buf,
        );
    }
}
