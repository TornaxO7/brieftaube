mod action;
mod state;

use crate::{
    backend::Account,
    ui::{MailId, keybindmanager::KeybindManager, mail_viewer::state::RenderData, palette},
};
pub use action::Action;
use crossterm::event::KeyEvent;
use jmap_client::email::Email;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};
use std::{collections::HashMap, sync::Arc};
use tracing::debug;
use tui_widget_list::{ListBuilder, ListView};

#[derive(Debug, Clone)]
enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct MailViewer {
    state: state::State,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<super::Action>,
}

impl MailViewer {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            state: state::State::new(account),
            palette: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::ScrollLeft.into()),
                ("j", Action::ScrollDown.into()),
                ("k", Action::ScrollUp.into()),
                ("l", Action::ScrollRight.into()),
                ("q", Action::Quit.into()),
                ("<BS>", super::Action::OpenMailList(None)),
            ])),
        }
    }

    pub fn open_mail(&mut self, mail: Option<MailId>) {
        if let Some(mail) = mail {
            self.state.open_mail(mail);
        }
    }

    pub fn handle_event(&mut self, event: KeyEvent) -> Vec<super::Action> {
        if let Some(palette) = &mut self.palette {
            let mut actions = Vec::new();
            if let Some(result) = palette.handle_event(event) {
                actions.push(Action::CloseCommandPalette.into());

                match result {
                    palette::HandleEventResult::Cancel => {}
                    palette::HandleEventResult::Selected(value) => match value {
                        PaletteType::Action(action) => {
                            actions.push(action.into());
                        }
                    },
                };
            }

            return actions;
        }

        match self.keybindings.handle_event(event) {
            Some(action) => vec![action],
            None => vec![],
        }
    }

    pub fn apply_action(&mut self, a: Action) -> Option<super::Action> {
        debug!("Action: {}", a);
        match a {
            Action::Quit => return Some(super::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(action::palette_options()));
            }
            Action::CloseCommandPalette => self.palette = None,

            Action::ScrollUp => self.state.scroll_up(),
            Action::ScrollDown => self.state.scroll_down(),
            Action::ScrollLeft => self.state.scroll_left(),
            Action::ScrollRight => self.state.scroll_right(),

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

        if let Some(mut data) = self.state.get_render_data() {
            let [top, bottom] = if data.mail.has_attachment() {
                Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area)
            } else {
                [area, Rect::default()]
            };

            render_mail_content(&mut data, top, buf);
            render_attachment_list(&data.mail, bottom, buf);
        } else {
            Widget::render(
                Paragraph::new("Loading mail...").block(Block::bordered()),
                area,
                buf,
            );
        }

        if let Some(state) = &mut self.palette {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Clear.render(a, buf);
            StatefulWidget::render(palette::Palette::new(), a, buf, state);
        }
    }
}

/// Rendering implementations
// TODO: Respect the area size before scrolling.
//    If the whole mail can be fitted within the area rect, there's no need to add the scrollbars.
fn render_mail_content(data: &mut RenderData, area: Rect, buf: &mut Buffer) {
    Widget::render(
        Paragraph::new(data.mail_str)
            .block(Block::bordered())
            .scroll((
                data.vertical.get_position() as u16,
                data.horizontal.get_position() as u16,
            )),
        area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        }),
        buf,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        buf,
        data.vertical,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
        area,
        buf,
        data.horizontal,
    );
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
