use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Rect},
    style::{Style, palette::material::BLUE},
    text::Text,
    widgets::{Block, Row, StatefulWidget, Table, Widget},
};

use crate::mail_viewer::types::MailDisplay;

pub struct MetadataViewer<'a> {
    pub mail: &'a MailDisplay,
}

impl<'a> StatefulWidget for MetadataViewer<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, _model: &mut Self::State) {
        const HEADERS: [&str; 6] = [
            "Received at:",
            "From:",
            "To:",
            "Subject:",
            "Cc:",
            "Keywords:",
        ];
        const LONGEST_HEADER_NAME: usize = longest_header_name(&HEADERS);

        let rows: Vec<Row> = [
            (HEADERS[0], &self.mail.received_at),
            (HEADERS[1], &self.mail.from),
            (HEADERS[2], &self.mail.to),
            (HEADERS[3], &self.mail.subject),
            (HEADERS[4], &self.mail.cc),
            (HEADERS[5], &self.mail.keywords),
        ]
        .iter()
        .map(|(header, value)| {
            let header = Text::raw(*header)
                .alignment(HorizontalAlignment::Right)
                .style(Style::new().fg(BLUE.c500));

            let value = Text::raw(*value);

            Row::new([header, value])
        })
        .collect();

        let widths = [
            Constraint::Length(LONGEST_HEADER_NAME as u16),
            Constraint::Fill(1),
        ];

        Widget::render(
            Table::new(rows, widths).block(Block::bordered().title("Metadata")),
            area,
            buf,
        );
    }
}

// I wish `const` in rust would be more powerful :(
const fn longest_header_name(headers: &[&str]) -> usize {
    let mut max = 0;
    let mut i = 0;
    while i < headers.len() {
        if headers[i].len() > max {
            max = headers[i].len();
        }
        i += 1;
    }
    max
}
