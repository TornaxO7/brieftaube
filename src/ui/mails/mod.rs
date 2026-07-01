mod action;
mod mail_list;
mod state;

use crate::{
    backend,
    ui::{
        command_palette::{self, CommandPalette},
        mails::mail_list::MailListWidget,
    },
};
pub use action::Action;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use state::State;
use std::{str::FromStr, sync::Arc};

const MAIL_LIST_PANEL_TITLE: &str = "Mails";
const PREVIEW_PANEL_TITLE: &str = "Mail content";

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

    pub fn open_mailbox(&mut self, mailbox_id: super::MailboxId) {
        self.state.open_mailbox(mailbox_id);
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

            Action::SelectNextMail => self.state.select_next_mail(),
            Action::SelectPreviousMail => self.state.select_previous_mail(),

            Action::OpenMailboxList => {
                return Some(super::Action::OpenMailboxList);
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
        let [headerbar, content] = area.layout(&Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(0),
        ]));

        let [mail_list, preview] = content.layout(&Layout::horizontal([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]));

        self.render_mail_list(mail_list, buf);
        self.render_preview(preview, buf);
        self.render_headerbar(headerbar, buf);

        if let Some(command_palette) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            command_palette.palette.render(a, buf);
        }
    }
}

/// Render functions
impl Mails {
    fn render_mail_list(&mut self, area: Rect, buf: &mut Buffer) {
        match self.state.get_mails() {
            Some(names) => StatefulWidget::render(
                MailListWidget::new(&names).block(
                    Block::bordered()
                        .title(MAIL_LIST_PANEL_TITLE)
                        .title_alignment(HorizontalAlignment::Center),
                ),
                area,
                buf,
                self.state.get_mail_list_state_mut(),
            ),
            None => Widget::render(
                Paragraph::new("Loading...").block(Block::bordered().title(MAIL_LIST_PANEL_TITLE)),
                area,
                buf,
            ),
        }
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(selected_mail_idx) = self.state.selected_mail_list_idx() {
            if let Some(mail) = self.state.get_mail(selected_mail_idx) {
                Widget::render(
                    Paragraph::new(mail.preview().unwrap())
                        .block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
                    area,
                    buf,
                );
                return;
            }
        }

        Widget::render(
            Paragraph::new("No mail selected").block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
            area,
            buf,
        );
    }

    fn render_headerbar(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered();
        let header_area = block.inner(area);

        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Fill(0),
            Constraint::Fill(0),
        ])
        .areas(header_area);

        Widget::render(block, area, buf);
        Widget::render(
            Paragraph::new("Left").alignment(HorizontalAlignment::Left),
            left,
            buf,
        );
        Widget::render(
            Paragraph::new("Center").alignment(HorizontalAlignment::Center),
            center,
            buf,
        );
        Widget::render(
            Paragraph::new("Right").alignment(HorizontalAlignment::Right),
            right,
            buf,
        );
    }
}
