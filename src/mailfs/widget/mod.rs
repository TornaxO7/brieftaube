mod column_data;
mod mail_preview;
mod render_data;

pub use column_data::{ColumnDisplay, ColumnDisplayEntryData, MailEntryType, RightColumn};
pub use mail_preview::MailPreview;
pub use render_data::RenderData;

use super::State;
use crate::utils::ui::ScreenState;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, BLUE_GRAY, CYAN, GRAY, GREEN, LIGHT_BLUE, ORANGE, PINK, WHITE},
    },
    symbols::line::VERTICAL,
    text::Text,
    widgets::{Block, Cell, List, ListItem, Paragraph, Row, StatefulWidget, Table, Widget},
};

const FOLDER: &str = "🖿";
const PLACEHOLDER: &str = "";
const MAIL_UNREAD_SYMBOL: &str = "⏺";
const THREAD_BRANCH: &str = "├─";
const THREAD_LAST: &str = "╰─";
const THREAD_FOLDED: &str = "▸";
const THREAD_UNFOLDED: &str = "▾";
const ATTACHMENT: &str = "📎";

#[derive(Default)]
pub struct Mailfs {}

impl StatefulWidget for Mailfs {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut data = state.render_data();

        let [
            left_area,
            border_line1,
            center_area,
            border_line2,
            right_area,
        ] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(1),
            Constraint::Fill(2),
        ])
        .areas(area);

        if let Some(left) = data.left.as_mut() {
            render_column(left_area, buf, left);
        }

        if let Some(center) = data.center.as_mut() {
            render_column(center_area, buf, center);
        }
        render_left_border_line(border_line1, buf, data.center.as_ref());

        if let Some(right) = data.right.as_mut() {
            render_right_column(right_area, buf, right);
        }
        render_left_border_line(border_line2, buf, None);
    }
}

fn render_column(area: Rect, buf: &mut Buffer, data: &mut ColumnDisplay) {
    let widths = [
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
    ];

    let rows: Vec<Row<'_>> = data
        .entries
        .iter()
        .map(|entry| {
            let mut row = Vec::with_capacity(widths.len());

            match &entry.data {
                ColumnDisplayEntryData::Mailbox { name, unread_mails } => {
                    // ⏺
                    if *unread_mails > 0 {
                        row.push(Cell::from(MAIL_UNREAD_SYMBOL).style(Style::new().fg(BLUE.c800)));
                    } else {
                        row.push(PLACEHOLDER.into());
                    };

                    // 🖿
                    row.push(Cell::from(FOLDER).style(Style::new().fg(BLUE.c500)));

                    // name
                    row.push(
                        Cell::from(name.clone())
                            .style(Style::new().fg(LIGHT_BLUE.c600))
                            .column_span(2),
                    );

                    // unread_mails
                    {
                        let style = if *unread_mails == 0 {
                            Style::new().fg(GRAY.c600)
                        } else {
                            Style::new().fg(GREEN.c500)
                        };

                        row.push(Cell::from(format!("{}", unread_mails)).style(style));
                    }
                }
                ColumnDisplayEntryData::Mail {
                    ty,
                    from,
                    subject,
                    received_at,
                    has_attachment,
                    is_unread,
                } => {
                    // ⏺
                    if *is_unread {
                        row.push(Cell::from(MAIL_UNREAD_SYMBOL).style(Style::new().fg(BLUE.c800)));
                    } else {
                        row.push(PLACEHOLDER.into());
                    };

                    // ▸/▾
                    match ty {
                        MailEntryType::Single
                        | MailEntryType::ThreadChild
                        | MailEntryType::ThreadEnd => {
                            row.push(Cell::from(PLACEHOLDER));
                        }
                        MailEntryType::ThreadCollapsed => row.push(Cell::from(THREAD_FOLDED)),
                        MailEntryType::ThreadStart => row.push(Cell::from(THREAD_UNFOLDED)),
                    };

                    // Subject
                    {
                        // 📎
                        let subject = if *has_attachment {
                            format!("{} {}", ATTACHMENT, subject)
                        } else {
                            subject.to_string()
                        };

                        let subject = match ty {
                            MailEntryType::Single
                            | MailEntryType::ThreadCollapsed
                            | MailEntryType::ThreadStart => subject,
                            MailEntryType::ThreadChild => {
                                format!("{} {}", THREAD_BRANCH, subject)
                            }
                            MailEntryType::ThreadEnd => {
                                format!("{} {}", THREAD_LAST, subject)
                            }
                        };

                        row.push(Cell::from(subject).style(Style::new().fg(WHITE)));
                    }

                    // from
                    row.push(Cell::from(from.clone()).style(Style::new().fg(CYAN.c800)));

                    // received at
                    row.push(Cell::from(received_at.clone()).style(Style::new().fg(PINK.c800)));
                }
            };

            Row::new(row)
        })
        .collect();

    StatefulWidget::render(
        Table::new(rows, widths)
            .row_highlight_style(Style::new().bg(LIGHT_BLUE.c500).fg(GRAY.c900)),
        area,
        buf,
        data.state,
    );
}

fn render_right_column(area: Rect, buf: &mut Buffer, data: &mut RightColumn) {
    match data {
        RightColumn::ColumnData(data) => render_column(area, buf, data),
        RightColumn::MailPreview(preview) => render_mail_preview(area, buf, preview),
    }
}

fn render_mail_preview(area: Rect, buf: &mut Buffer, mail: &mut MailPreview) {
    const HEADERS: [&str; 5] = ["Received at:", "From:", "To:", "Subject:", "Cc:"];

    let [header_area, preview_area] = Layout::vertical([
        Constraint::Length(HEADERS.len() as u16 + 2),
        Constraint::Fill(1),
    ])
    .areas(area);

    let MailPreview {
        from,
        to,
        cc,
        subject,
        preview,
        received_at,
    } = mail;

    let headers: Vec<(&str, &str)> = HEADERS
        .iter()
        .zip([
            received_at.as_str(),
            from.as_str(),
            to.as_str(),
            subject.as_str(),
            cc.as_str(),
        ])
        .map(|(&header_name, value)| (header_name, value))
        .collect();

    render_headers(header_area, buf, &headers);

    Widget::render(
        Paragraph::new(preview.as_str()).block(Block::bordered()),
        preview_area,
        buf,
    );
}

fn render_headers(area: Rect, buf: &mut Buffer, headers: &[(&'static str, &str)]) {
    let table = {
        let rows: Vec<ratatui::widgets::Row<'_>> = headers
            .iter()
            .map(|(name, value)| {
                ratatui::widgets::Row::new([
                    Cell::new(Text::from(*name).right_aligned())
                        .style(Style::default().fg(BLUE_GRAY.c400)),
                    Cell::new(*value),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(
                headers
                    .iter()
                    .map(|(header, _)| header.len())
                    .max()
                    .unwrap_or(5) as u16,
            ),
            Constraint::Fill(1),
        ];

        Table::new(rows, widths).block(Block::bordered())
    };

    Widget::render(table, area, buf);
}

fn render_left_border_line(area: Rect, buf: &mut Buffer, column: Option<&ColumnDisplay>) {
    match column {
        None => Widget::render(List::new(vec![VERTICAL; area.height as usize]), area, buf),
        Some(column) => {
            let mut lines: Vec<ListItem> = column
                .entries
                .iter()
                .map(|entry| {
                    if entry.is_selected {
                        ListItem::new(VERTICAL).style(Style::new().bg(ORANGE.c500))
                    } else {
                        ListItem::new(VERTICAL)
                    }
                })
                .collect();

            lines.resize(area.height as usize, ListItem::new(VERTICAL));

            Widget::render(List::new(lines), area, buf)
        }
    }
}
