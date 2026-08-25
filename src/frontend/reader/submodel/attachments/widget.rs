use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, GREEN, PINK, YELLOW},
    },
    text::Text,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};
use throbber_widgets_tui::Throbber;

use crate::{
    backend::types::Loadable,
    reader::{model::attachments::Navigate, types::MailDisplayAttachment},
};

pub struct AttachmentsReader<'a> {
    pub attachments: &'a mut Loadable<Vec<MailDisplayAttachment>>,
}

impl<'a> StatefulWidget for AttachmentsReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        match self.attachments {
            Loadable::NotRequested => {
                Widget::render(Paragraph::new("Not requested yet"), area, buf);
            }
            Loadable::Requested { .. } => StatefulWidget::render(
                Throbber::default()
                    .label("Fetching attachments of mail...")
                    .throbber_set(throbber_widgets_tui::BRAILLE_SIX),
                area,
                buf,
                &mut model.throbber,
            ),
            Loadable::Loaded(attachments) => {
                // if !attachments.is_empty() && model.state.selected().is_none() {
                //     model.state.select(Some(0));
                // }

                if let Some(navigate) = model.navigate.take() {
                    apply_navigation(&mut model.state, navigate, attachments, area);
                }

                let longest_name = attachments
                    .iter()
                    .map(|attachment| attachment.name.len())
                    .max()
                    .unwrap_or(0);

                let longest_size_length = MailDisplayAttachment::MAX_DISPLAY_LENGTH;

                let rows: Vec<Row<'_>> = attachments
                    .iter()
                    .map(|a| {
                        let downloaded_symbol = if a.downloaded {
                            Text::raw("✓").style(Style::new().fg(GREEN.c500))
                        } else {
                            Text::raw("↓").style(Style::new().fg(BLUE.c500).bold())
                        };

                        Row::new([
                            downloaded_symbol,
                            Text::raw(&a.size).style(Style::new().fg(YELLOW.c500)),
                            Text::raw(&a.name),
                            Text::raw(&a.content_type).style(Style::new().fg(PINK.c500)),
                        ])
                    })
                    .collect();

                let widths = [
                    Constraint::Length(1),
                    Constraint::Length(longest_size_length),
                    Constraint::Length(longest_name as u16),
                    Constraint::Fill(1),
                ];

                StatefulWidget::render(
                    Table::new(rows, widths)
                        .row_highlight_style(Style::new().fg(BLACK).bg(BLUE.c500))
                        .block(Block::bordered().title("Attachments"))
                        .column_spacing(3),
                    area,
                    buf,
                    &mut model.state,
                );
            }
        }
    }
}

fn apply_navigation(
    state: &mut TableState,
    navigate: Navigate,
    attachments: &[MailDisplayAttachment],
    area: Rect,
) {
    match navigate {
        Navigate::Up(amount) => {
            state.scroll_up_by(amount);
        }
        Navigate::Down(amount) => {
            state.scroll_down_by(amount);
        }
        Navigate::HalfPageUp => {
            state.scroll_up_by(area.height);
        }
        Navigate::HalfPageDown => state.scroll_down_by(area.height),
        Navigate::Top => {
            if !attachments.is_empty() {
                state.select(Some(0));
            }
        }
        Navigate::Bottom => {
            if !attachments.is_empty() {
                state.select(Some(attachments.len() - 1));
            }
        }
    }
}
