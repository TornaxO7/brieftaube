use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, PINK, YELLOW},
    },
    text::Text,
    widgets::{Block, ListState, Row, StatefulWidget, Table, TableState},
};

use crate::mail_viewer::{model::attachments::Navigate, types::MailDisplayAttachment};

pub struct AttachmentsViewer<'a> {
    pub attachments: &'a [MailDisplayAttachment],
}

impl<'a> StatefulWidget for AttachmentsViewer<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        if let Some(navigate) = model.navigate.take() {
            apply_navigation(&mut model.state, navigate, &self.attachments, area);
        }

        let rows: Vec<Row<'_>> = self
            .attachments
            .iter()
            .map(|a| {
                Row::new([
                    Text::raw(&a.size).style(Style::new().fg(YELLOW.c500)),
                    Text::raw(&a.name),
                    Text::raw(&a.content_type).style(Style::new().fg(PINK.c500)),
                ])
            })
            .collect();

        let widths = [Constraint::Max(6), Constraint::Fill(1), Constraint::Fill(1)];

        StatefulWidget::render(
            Table::new(rows, widths)
                .row_highlight_style(Style::new().fg(BLACK).bg(BLUE.c500))
                .block(Block::bordered().title("Attachments")),
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
