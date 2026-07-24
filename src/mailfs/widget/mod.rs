mod column_data;
mod render_data;

pub use column_data::ColumnData;
pub use render_data::RenderData;

use crate::{
    mailfs::widget::column_data::{ColumnEntryData, MailEntryType},
    utils::ui::ScreenState,
};

use super::State;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        Style,
        palette::material::{BLUE, CYAN, GRAY, ORANGE, PINK, WHITE},
    },
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
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
        if let Some(mut data) = state.render_data() {
            let [left_area, center_area, right_area] = Layout::horizontal([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .areas(area);

            if let Some(left) = &mut data.left {
                render_column(left_area, buf, left);
            }
            render_line(left_area, buf);
            render_column(center_area, buf, &mut data.center);
            render_line(center_area, buf);
            if let Some(right) = &mut data.right {
                render_column(right_area, buf, right);
            }
        } else {
            // loading screen
        }
    }
}

fn render_column(area: Rect, buf: &mut Buffer, data: &mut ColumnData) {
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
                    thread,
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
                                ColumnEntryData::Mail {
                                    thread: other_thread,
                                    ty: other_ty,
                                    ..
                                } => {
                                    let is_child = *other_ty == MailEntryType::ThreadChild;
                                    let in_same_thread = other_thread == thread;

                                    if is_child && in_same_thread {
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
                    row.push(Cell::from(*from).style(Style::new().fg(CYAN.c800)));

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

fn render_line(area: Rect, buf: &mut Buffer) {
    Block::new().borders(Borders::RIGHT).render(area, buf);
}
