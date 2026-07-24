mod column_data;
mod mail_preview;
mod render_data;

pub use column_data::ColumnData;
pub use render_data::RenderData;

use crate::{
    backend::mails::types::MailPreview,
    mailfs::widget::column_data::{ColumnEntryData, MailEntryType, RightColumn},
    utils::ui::ScreenState,
};

use super::State;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, BLUE_GRAY, CYAN, GRAY, ORANGE, PINK, WHITE},
    },
    text::Text,
    widgets::{Block, Borders, Cell, Paragraph, Row, StatefulWidget, Table, Widget},
};

const FOLDER: &str = "🗀";
const FOLDER_OPEN: &str = "🗁";
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

        let [left_area, center_area, right_area] = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .areas(area);

        render_column(left_area, buf, data.left.as_mut());
        render_line(left_area, buf);
        render_column(center_area, buf, data.center.as_mut());
        render_line(center_area, buf);
        render_right_column(right_area, buf, &mut data.right);
    }
}

fn render_column(area: Rect, buf: &mut Buffer, data: Option<&mut ColumnData>) {
    let Some(data) = data else {
        todo!("Render loading screen");
    };

    let widths = [
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Fill(2),
    ];

    let rows: Vec<Row<'_>> = data
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let is_hovered = data
                .state
                .selected()
                .map(|hovered_idx| hovered_idx == idx)
                .unwrap_or(false);

            let mut row = Vec::with_capacity(widths.len());

            if entry.is_selected {
                row.push(Cell::from(PLACEHOLDER).style(Style::new().bg(ORANGE.c800)));
            } else {
                row.push(PLACEHOLDER.into());
            };

            match &entry.data {
                ColumnEntryData::Mailbox { name, unread_mails } => {
                    // 🗀 / 🗁
                    {
                        let s = if is_hovered { FOLDER_OPEN } else { FOLDER };
                        row.push(
                            Cell::from(s)
                                .style(Style::new().fg(BLUE.c800))
                                .column_span(3),
                        );
                    }

                    // name
                    row.push(
                        Cell::from(*name)
                            .style(Style::new().fg(BLUE.c800))
                            .column_span(2),
                    );

                    // unread_mails
                    {
                        let style = if *unread_mails == 0 {
                            Style::new().fg(GRAY.c800)
                        } else {
                            Style::new().fg(BLUE.c800)
                        };

                        row.push(Cell::from(format!("{}", unread_mails)).style(style));
                    }
                }
                ColumnEntryData::Mail {
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
                        MailEntryType::Single | MailEntryType::ThreadChild => {
                            row.push(Cell::from(PLACEHOLDER));
                        }
                        MailEntryType::ThreadRoot => match data.entries.get(idx + 1) {
                            Some(next_entry) => match &next_entry.data {
                                ColumnEntryData::Mailbox { .. } => {
                                    row.push(Cell::from(PLACEHOLDER))
                                }
                                ColumnEntryData::Mail { ty: other_ty, .. } => {
                                    if *other_ty == MailEntryType::ThreadChild {
                                        row.push(Cell::from(THREAD_UNFOLDED));
                                    } else {
                                        row.push(Cell::from(THREAD_FOLDED));
                                    }
                                }
                            },
                            None => {
                                row.push(Cell::from(PLACEHOLDER));
                            }
                        },
                    };

                    // 📎
                    if *has_attachment {
                        row.push(Cell::from(ATTACHMENT));
                    } else {
                        row.push(Cell::from(PLACEHOLDER));
                    }

                    // Subject
                    {
                        let s = match ty {
                            MailEntryType::Single | MailEntryType::ThreadRoot => {
                                subject.to_string()
                            }
                            MailEntryType::ThreadChild => match data.entries.get(idx + 1) {
                                Some(next_entry) => match next_entry.data {
                                    ColumnEntryData::Mail { ty, .. } => {
                                        if ty == MailEntryType::ThreadChild {
                                            format!("{} {}", THREAD_BRANCH, subject)
                                        } else {
                                            format!("{} {}", THREAD_LAST, subject)
                                        }
                                    }
                                    ColumnEntryData::Mailbox { .. } => {
                                        format!("{} {}", THREAD_LAST, subject)
                                    }
                                },
                                None => {
                                    format!("{} {}", THREAD_LAST, subject)
                                }
                            },
                        };

                        row.push(Cell::from(s).style(Style::new().fg(WHITE)));
                    }

                    // from
                    row.push(Cell::from(from.clone()).style(Style::new().fg(CYAN.c800)));

                    // received at
                    row.push(Cell::from(*received_at).style(Style::new().fg(PINK.c800)));
                }
            };

            Row::new(row)
        })
        .collect();

    StatefulWidget::render(
        Table::new(rows, widths).row_highlight_style(Style::new().bg(BLUE.c600)),
        area,
        buf,
        data.state,
    );
}

fn render_right_column(area: Rect, buf: &mut Buffer, data: &mut RightColumn) {
    match data {
        RightColumn::ColumnData(data) => render_column(area, buf, data.as_mut()),
        RightColumn::MailPreview(preview) => render_mail_preview(area, buf, preview.as_mut()),
    }
}

fn render_mail_preview(area: Rect, buf: &mut Buffer, mail: Option<&mut MailPreview>) {
    let Some(mail) = mail else {
        todo!("Render loading screen");
    };

    let headers = vec![
        ("Received at:", mail.received_at.as_str()),
        ("From:", mail.from.as_str()),
        ("To:", mail.to.as_str()),
        ("Subject:", mail.subject.as_str()),
        ("Cc:", mail.cc.as_str()),
    ];

    let [header_area, preview_area] = Layout::vertical([
        Constraint::Length(headers.len() as u16 + 2),
        Constraint::Fill(1),
    ])
    .areas(area);

    render_headers(header_area, buf, &headers);

    Widget::render(
        Paragraph::new(mail.preview.as_str()).block(Block::bordered()),
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

fn render_line(area: Rect, buf: &mut Buffer) {
    Block::new().borders(Borders::RIGHT).render(area, buf);
}
