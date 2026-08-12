mod column_data;
mod error;
mod mail_preview;
mod render_data;
mod selection_type;

pub use column_data::{ColumnDisplay, ColumnDisplayEntryData, MailEntryType, RightColumn};
pub use mail_preview::MailPreview;
pub use render_data::RenderData;

use super::State;
use crate::{
    mailfs::widget::selection_type::DisplaySelectionType,
    utils::ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette},
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::{
        Style,
        palette::material::{
            BLUE, BLUE_GRAY, CYAN, GRAY, GREEN, LIGHT_BLUE, ORANGE, PINK, RED, TEAL, WHITE, YELLOW,
        },
    },
    symbols::line::VERTICAL,
    text::{Line, Text},
    widgets::{Block, Cell, Clear, List, ListItem, Paragraph, Row, StatefulWidget, Table, Widget},
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

        let [path_area, filesystem_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        render_path(path_area, buf, &data.mailbox_path);

        let [
            border_line1,
            left_area,
            border_line2,
            center_area,
            border_line3,
            right_area,
        ] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(1),
            Constraint::Fill(2),
        ])
        .areas(filesystem_area);

        render_border_line(border_line1, buf, data.left.as_ref());
        if let Some(left) = data.left.as_mut() {
            render_column(left_area, buf, left);
        }
        if let Some(center) = data.center.as_mut() {
            render_column(center_area, buf, center);
        }
        render_border_line(border_line2, buf, data.center.as_ref());

        if let Some(right) = data.right.as_mut() {
            render_right_column(right_area, buf, right);
        }
        render_border_line(
            border_line3,
            buf,
            data.right.as_ref().and_then(|right| match right {
                RightColumn::ColumnData(column) => Some(column),
                RightColumn::MailPreview(_) => None,
            }),
        );

        if let Some(overlay) = state.overlay() {
            match overlay {
                ScreenOverlay::Palette(state) => {
                    let area =
                        area.centered(Constraint::Percentage(80), Constraint::Percentage(80));
                    Widget::render(Clear, area, buf);
                    StatefulWidget::render(Palette::new(), area, buf, state)
                }
                ScreenOverlay::Input(state) => {
                    let area = area.centered(
                        Constraint::Length(state.input_len(0).max(75) as u16),
                        Constraint::Length(3),
                    );
                    Widget::render(Clear, area, buf);
                    StatefulWidget::render(Input::new(), area, buf, state)
                }
            }
        }
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
                ColumnDisplayEntryData::Mailbox {
                    name,
                    unread_mails,
                    sort_order,
                } => {
                    // ⏺
                    if *unread_mails > 0 {
                        row.push(Cell::from(MAIL_UNREAD_SYMBOL).style(Style::new().fg(BLUE.c800)));
                    } else {
                        row.push(PLACEHOLDER.into());
                    };

                    // 🖿
                    row.push(Cell::from(FOLDER).style(Style::new().fg(BLUE.c500)));

                    // name
                    row.push(Cell::from(name.clone()).style(Style::new().fg(LIGHT_BLUE.c600)));

                    // unread_mails
                    {
                        let style = if *unread_mails == 0 {
                            Style::new().fg(GRAY.c600)
                        } else {
                            Style::new().fg(GREEN.c500)
                        };

                        row.push(Cell::from(format!("{}", unread_mails)).style(style));
                    }

                    // sort-order
                    row.push(Cell::from(format!("{}", sort_order)));
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

    let MailPreview {
        from,
        to,
        cc,
        subject,
        preview,
        received_at,
        attachments,
    } = mail;

    let (header_area, preview_area, attachments_area) = {
        let mut constraints = vec![
            Constraint::Length(HEADERS.len() as u16 + 2),
            Constraint::Fill(1),
        ];

        // TODO: Check if attachments are currently _loading_ or not...
        //       To avoid a sudden appearence of the attachment list
        match attachments {
            Some(a) => {
                if a.is_empty() {
                    let [header, preview] = Layout::vertical(constraints).areas(area);
                    (header, preview, None)
                } else {
                    constraints.push(Constraint::Max(a.len() as u16 + 2));

                    let [header, preview, attachments_area] =
                        Layout::vertical(constraints).areas(area);
                    (header, preview, Some(attachments_area))
                }
            }
            None => {
                constraints.push(Constraint::Length(3));
                let [header, preview, attachments_area] = Layout::vertical(constraints).areas(area);
                (header, preview, Some(attachments_area))
            }
        }
    };

    // headers
    {
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
    }

    // preview
    Widget::render(
        Paragraph::new(preview.as_str()).block(Block::bordered().title("Preview")),
        preview_area,
        buf,
    );

    // attachments
    if let Some(area) = attachments_area {
        let block = Block::bordered().title("Attachments");
        match attachments {
            Some(attachments) => {
                let rows: Vec<Row> = attachments
                    .iter()
                    .map(|attachment| {
                        Row::new([
                            Cell::new(attachment.name.clone()),
                            Text::raw(attachment.content_type.clone())
                                .alignment(HorizontalAlignment::Right)
                                .into(),
                        ])
                    })
                    .collect();

                let widths = [Constraint::Fill(1), Constraint::Fill(1)];

                Widget::render(Table::new(rows, widths).block(block), area, buf);
            }
            None => {
                Widget::render(
                    Paragraph::new("Fetching attachments...")
                        .style(Style::new().fg(YELLOW.c500))
                        .block(block),
                    area,
                    buf,
                );
            }
        };
    }
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

fn render_border_line(area: Rect, buf: &mut Buffer, column: Option<&ColumnDisplay>) {
    match column {
        None => Widget::render(List::new(vec![VERTICAL; area.height as usize]), area, buf),
        Some(column) => {
            let mut lines: Vec<ListItem> = column
                .entries
                .iter()
                .map(|entry| {
                    let style = entry
                        .selection_type
                        .map(|ty| match ty {
                            DisplaySelectionType::Selected => Style::new().bg(ORANGE.c500),
                            DisplaySelectionType::Cut => Style::new().bg(RED.c500),
                        })
                        .unwrap_or(Style::new());

                    ListItem::new(VERTICAL).style(style)
                })
                .collect();

            lines.resize(area.height as usize, ListItem::new(VERTICAL));

            Widget::render(List::new(lines), area, buf)
        }
    }
}

fn render_path(area: Rect, buf: &mut Buffer, path: &str) {
    Widget::render(Line::raw(path).style(Style::new().fg(TEAL.c400)), area, buf);
}
