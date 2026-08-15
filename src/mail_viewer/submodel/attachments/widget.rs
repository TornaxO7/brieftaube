use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{
        Style,
        palette::material::{BLACK, BLUE, PINK, YELLOW},
    },
    text::Text,
    widgets::{Block, Row, StatefulWidget, Table},
};

use crate::mail_viewer::{model::attachments::Navigate, types::MailDisplayAttachment};

pub struct AttachmentsViewer<'a> {
    pub attachments: &'a [MailDisplayAttachment],
}

impl<'a> StatefulWidget for AttachmentsViewer<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        if let Some(navigate) = model.navigate.take() {
            match navigate {
                Navigate::Up(amount) => {
                    model.state.scroll_up_by(amount);
                }
                Navigate::Down(amount) => {
                    model.state.scroll_down_by(amount);
                }
                Navigate::HalfPageUp => {
                    model.state.scroll_up_by(area.height);
                }
                Navigate::HalfPageDown => model.state.scroll_down_by(area.height),
                Navigate::Top => {
                    if !self.attachments.is_empty() {
                        model.state.select(Some(0));
                    }
                }
                Navigate::Bottom => {
                    if !self.attachments.is_empty() {
                        model.state.select(Some(self.attachments.len() - 1));
                    }
                }
            }
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
