use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, PINK, YELLOW},
    },
    text::Text,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, TableState, Widget},
};

use crate::reader::{model::attachments::Navigate, types::MailDisplayAttachment};

pub struct AttachmentsReader<'a> {
    pub attachments: Option<&'a Vec<MailDisplayAttachment>>,
}

impl<'a> StatefulWidget for AttachmentsReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let Some(attachments) = self.attachments else {
            Widget::render(Paragraph::new("Loading attachments"), area, buf);
            return;
        };

        if !attachments.is_empty() && model.state.selected().is_none() {
            model.state.select(Some(0));
        }

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
                Row::new([
                    Text::raw(&a.size).style(Style::new().fg(YELLOW.c500)),
                    Text::raw(&a.name),
                    Text::raw(&a.content_type).style(Style::new().fg(PINK.c500)),
                ])
            })
            .collect();

        let widths = [
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
