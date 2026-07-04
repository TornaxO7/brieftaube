mod action;
mod state;

use crate::{
    backend::Account,
    ui::{
        MailId,
        command_palette::{CommandPalette, HandleEventResult},
    },
};
pub use action::Action;
use chrono::DateTime;
use crossterm::event::{KeyCode, KeyEvent};
use jmap_client::email::{Email, EmailAddress};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use std::{str::FromStr, sync::Arc};
use tui_widget_list::{ListBuilder, ListView};

#[derive(Debug)]
enum PaletteType {
    /// Palette is displaying commands
    Command,
}

#[derive(Debug)]
struct PaletteCtx {
    palette: CommandPalette,
    ty: PaletteType,
}
#[derive(Debug)]
pub struct MailViewer {
    state: state::State,
    palette: Option<PaletteCtx>,
}

impl MailViewer {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            state: state::State::new(account),
            palette: None,
        }
    }

    pub fn open_mail(&mut self, mail: Option<MailId>) {
        if let Some(mail) = mail {
            self.state.open_mail(mail);
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        let mut actions = Vec::new();

        if let Some(command_palette) = &mut self.palette {
            if let Some(result) = command_palette.palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    HandleEventResult::Cancel => {}
                    HandleEventResult::Selected(value) => match command_palette.ty {
                        PaletteType::Command => {
                            actions.push(Action::from_str(&value).unwrap().into())
                        }
                    },
                };
            }

            return actions;
        }

        match event.code {
            KeyCode::Char('j') => actions.push(Action::ScrollDown.into()),
            KeyCode::Char('k') => actions.push(Action::ScrollUp.into()),
            KeyCode::Char('q') => actions.push(super::Action::Quit),
            KeyCode::Char('h') => actions.push(super::Action::OpenMailList(None)),
            _ => {}
        };

        actions
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(PaletteCtx {
                    palette: CommandPalette::new(Action::palette_options()),
                    ty: PaletteType::Command,
                })
            }
            Action::CloseCommandPalette => self.palette = None,
            Action::ScrollUp => self.state.scroll_up(),
            Action::ScrollDown => self.state.scroll_down(),

            Action::OpenMailList => return Some(super::Action::OpenMailList(None)),
        }

        None
    }
}

impl Widget for &mut MailViewer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.state.update();

        if let Some(mail) = &self.state.get_mail() {
            let [top, bottom] = if mail.has_attachment() {
                Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area)
            } else {
                [area, Rect::default()]
            };

            render_mail_content(mail, top, buf);
            render_attachment_list(mail, bottom, buf);
        } else {
            Widget::render(
                Paragraph::new("Loading mail...").block(Block::bordered()),
                area,
                buf,
            );
        }

        if let Some(cmd) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            cmd.palette.render(a, buf);
        }
    }
}

/// Rendering implementations
fn render_mail_content(mail: &Email, area: Rect, buf: &mut Buffer) {
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

            s.push_str(mail.body_value(id).unwrap().value());
        }

        s
    };

    let content = format!(
        "\
Date: {date}
From: {from}
To: {to}
Subject: {subject}


{body}"
    );

    Widget::render(Paragraph::new(content).block(Block::bordered()), area, buf)
}

fn render_attachment_list(mail: &Email, area: Rect, buf: &mut Buffer) {
    if let Some(attachments) = mail.attachments() {
        let builder = ListBuilder::new(|context| {
            const HEIGHT: u16 = 1;

            let attachment = &attachments[context.index];

            let widget = AttachmentWidget {
                name: attachment.name().unwrap(),
                ty: attachment.content_type().unwrap(),
            };

            (widget, HEIGHT)
        });

        let list =
            ListView::new(builder, attachments.len()).block(Block::bordered().title("Attachments"));

        StatefulWidget::render(list, area, buf, &mut tui_widget_list::ListState::default());
    }
}

fn addresses_to_string(addresses: &[EmailAddress]) -> String {
    let mut iter = addresses.iter();

    let mut s = format!("{}", iter.next().unwrap().email());

    for addr in iter {
        s.push_str(&format!(", {}", addr.email()))
    }

    s
}

#[derive(Debug)]
struct AttachmentWidget<'a> {
    name: &'a str,
    ty: &'a str,
}

impl<'a> Widget for AttachmentWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [left, right] =
            Layout::horizontal([Constraint::Fill(0), Constraint::Fill(0)]).areas(area);

        Widget::render(Paragraph::new(self.name).left_aligned(), left, buf);
        Widget::render(Paragraph::new(self.ty).right_aligned(), right, buf);
    }
}
