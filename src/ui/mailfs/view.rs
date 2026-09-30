use crate::{
    types::{MailId, MailKeyword, ParentMailboxId, ROOT_MAILBOX_ID, ThreadId},
    ui::{
        Loadable,
        mailfs::{
            ColumnStackEntry,
            columns::{MailboxColumnEntry, MailfsColumn},
        },
        statusbar::Statusbar,
    },
    utils::IntoColor,
};
use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Fill, Paragraph, Row, Table, Wrap},
};
use throbber_widgets_tui::Throbber;

const MAX_DATE_LENGTH: usize = "Jan 10, 1996".len();

const SEPARATION_LINE: &str = "│";
const PLACEHOLDER: &str = " ";

const FOLDER_ICON: &str = "🖿";
const MAIL_UNREAD_ICON: &str = "●";
const PAPERCLIP: &str = "📎";

pub fn view(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [path_area, columns_area, statusbar_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    render_path(scheme, state, frame, path_area);
    render_columns(scheme, state, frame, columns_area);
    render_statusbar(scheme, state, frame, statusbar_area);
}

fn render_path(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let mut path: Vec<Span> = Vec::new();

    for column_entry in state.column_stack.iter().cloned() {
        match column_entry {
            ColumnStackEntry::Users => {}
            ColumnStackEntry::Accounts => {
                let selected_user = &state.users_column.get_selected_entry().username;
                let account = &state
                    .accounts_column
                    .get(selected_user)
                    .expect("Account column exists")
                    .loaded()
                    .unwrap()
                    .get_selected_entry()
                    .name;

                path.push(Span::styled(
                    format!("[{}]:", account.as_str()),
                    Style::new().fg(scheme.primary.into_color()),
                ));
            }
            ColumnStackEntry::Mailbox(mailbox_id) => {
                let key = state.get_account_ctx().as_key(mailbox_id);
                let column = state.mailbox_columns.get(&key).unwrap();

                let Some(selected_entry) = column.get_selected_entry() else {
                    break;
                };

                match selected_entry {
                    Loadable::NotLoaded | Loadable::Loading | Loadable::Error(_) => {
                        break;
                    }
                    Loadable::Loaded(entry) => match entry {
                        MailboxColumnEntry::Mailbox(mailbox) => {
                            path.push(Span::styled(
                                format!("/{}", mailbox.name.as_str()),
                                Style::new().fg(scheme.secondary.into_color()),
                            ));
                        }
                        MailboxColumnEntry::RootMail(_mail) => {
                            continue;
                        }
                    },
                }
            }
            ColumnStackEntry::Thread(_thread_id) => {
                path.push(Span::styled(
                    "/<Thread>",
                    Style::new().fg(scheme.secondary.into_color()),
                ));
                continue;
            }
        }
    }

    frame.render_widget(
        Paragraph::new(Line::from(path)).block(
            Block::new()
                .borders(Borders::BOTTOM)
                .style(Style::new().fg(scheme.outline.into_color())),
        ),
        area,
    );
}

fn render_statusbar(_scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let layer_name = {
        let column_type = match state.column_stack.last().unwrap() {
            ColumnStackEntry::Users => "Users",
            ColumnStackEntry::Accounts => "Accounts",
            ColumnStackEntry::Mailbox(_) => "Mailbox",
            ColumnStackEntry::Thread(_) => "Thread",
        };

        let mode = match state.mode {
            super::Mode::Normal => "normal",
        };

        format!("{}({})", column_type, mode)
    };

    frame.render_widget(Statusbar::default().layer_name(layer_name.as_str()), area);
}

fn render_columns(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [
        _left_padding,
        left_area,
        sep1,
        middle_area,
        sep2,
        right_area,
    ] = Layout::horizontal([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    render_left_column(scheme, state, frame, left_area);
    render_left_separation_lines(scheme, frame, sep1);
    render_middle_column(scheme, state, frame, middle_area);
    render_left_separation_lines(scheme, frame, sep2);
    render_right_column(scheme, state, frame, right_area);
}

fn render_left_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let Some(prev_last) = state.column_stack.iter().rev().skip(1).next().cloned() else {
        return;
    };

    render_column(scheme, prev_last, state, frame, area);
}

fn render_middle_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let entry = state.column_stack.last().cloned().unwrap();
    render_column(scheme, entry, state, frame, area);
}

fn render_right_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    match state.column_stack.last().unwrap() {
        ColumnStackEntry::Users => {
            render_accounts_column(scheme, state, frame, area);
        }
        ColumnStackEntry::Accounts => {
            render_mailbox_column(scheme, ROOT_MAILBOX_ID, state, frame, area);
        }
        ColumnStackEntry::Mailbox(mailbox_id) => {
            let key = state.get_account_ctx().as_key(mailbox_id.clone());

            let column = state.mailbox_columns.get(&key).unwrap();
            let Some(selected_entry) = column.get_selected_entry() else {
                return;
            };

            match selected_entry {
                Loadable::NotLoaded => todo!("Maybe display some information?"),
                Loadable::Loading => {}
                Loadable::Error(err) => {
                    frame.render_widget(
                        Paragraph::new(format!("Couldn't load mailbox: {}", err))
                            .wrap(Wrap { trim: false })
                            .style(Style::new().fg(scheme.error.into_color())),
                        area,
                    );
                }
                Loadable::Loaded(entry) => match entry {
                    MailboxColumnEntry::Mailbox(mailbox_data) => {
                        render_mailbox_column(
                            scheme,
                            Some(mailbox_data.id.clone()),
                            state,
                            frame,
                            area,
                        );
                    }
                    MailboxColumnEntry::RootMail(mail_data_core) => {
                        render_thread_column(
                            scheme,
                            mail_data_core.thread_id.clone(),
                            state,
                            frame,
                            area,
                        );
                    }
                },
            }
        }
        ColumnStackEntry::Thread(thread_id) => {
            let thread_key = state.get_account_ctx().as_key(thread_id.clone());

            match state
                .thread_columns
                .get(&thread_key)
                .expect("Column exists")
            {
                Loadable::NotLoaded => unreachable!(),
                Loadable::Loading => {
                    frame.render_stateful_widget(
                        Throbber::default()
                            .throbber_style(Style::default().fg(scheme.primary.into_color())),
                        area,
                        &mut state.throbber,
                    );
                }
                Loadable::Loaded(column) => {
                    let mail_id = column.get_selected_entry().id.clone();
                    render_mail_preview(scheme, mail_id, state, frame, area);
                }
                Loadable::Error(err) => frame.render_widget(
                    Paragraph::new(format!("Couldn't load thread:\n{err}"))
                        .wrap(Wrap { trim: false })
                        .style(Style::new().fg(scheme.error.into_color())),
                    area,
                ),
            }
        }
    }
}

fn render_column(
    scheme: &Scheme,
    entry: ColumnStackEntry,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    match entry {
        ColumnStackEntry::Users => render_users_column(scheme, state, frame, area),
        ColumnStackEntry::Accounts => {
            render_accounts_column(scheme, state, frame, area);
        }
        ColumnStackEntry::Mailbox(mailbox_id) => {
            render_mailbox_column(scheme, mailbox_id, state, frame, area)
        }
        ColumnStackEntry::Thread(thread_id) => {
            render_thread_column(scheme, thread_id, state, frame, area)
        }
    }
}

fn render_users_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let rows: Vec<Row<'_>> = {
        let mut rows = Vec::new();

        for user in state.users_column.users.iter() {
            let row = Row::new([Cell::new(user.username.as_str())
                .style(Style::new().fg(scheme.secondary.into_color()))]);

            rows.push(row);
        }

        rows
    };

    let widths = [Constraint::Fill(1)];

    frame.render_stateful_widget(
        Table::new(rows, widths).row_highlight_style(
            Style::new()
                .bg(scheme.primary_container.into_color())
                .fg(scheme.on_primary_container.into_color()),
        ),
        area,
        &mut state.users_column.state,
    );
}

fn render_accounts_column(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let selected_user = &state.users_column.get_selected_entry().username;

    match state
        .accounts_column
        .get_mut(selected_user)
        .expect("Account column exists")
    {
        Loadable::NotLoaded => unreachable!(),
        Loadable::Loading => {
            frame.render_stateful_widget(
                Throbber::default().label("Login user..."),
                area,
                &mut state.throbber,
            );
        }
        Loadable::Error(err) => {
            frame.render_widget(
                Paragraph::new(format!("Couldn't connect to server:\n{}", err))
                    .wrap(Wrap { trim: false })
                    .style(Style::new().fg(scheme.error.into_color())),
                area,
            );
        }
        Loadable::Loaded(accounts) => {
            let rows: Vec<Row<'_>> = {
                let mut rows = Vec::with_capacity(accounts.len());

                for account in &accounts.accounts {
                    let row = Row::new([Cell::new(account.name.as_str())
                        .style(Style::new().fg(scheme.primary.into_color()))]);

                    rows.push(row);
                }

                rows
            };

            let widths = [Constraint::Fill(1)];

            frame.render_stateful_widget(
                Table::new(rows, widths).row_highlight_style(
                    Style::new()
                        .bg(scheme.primary_container.into_color())
                        .fg(scheme.on_primary_container.into_color()),
                ),
                area,
                &mut accounts.state,
            );
        }
    }
}

fn render_mailbox_column(
    scheme: &Scheme,
    mailbox_id: ParentMailboxId,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let key = state.get_account_ctx().as_key(mailbox_id);
    let mailbox_column = state.mailbox_columns.get_mut(&key).unwrap();

    let mailboxes_len = mailbox_column.mailboxes_len();
    let [mailbox_area, mails_area] = Layout::vertical([
        Constraint::Length(mailboxes_len.saturating_sub(mailbox_column.mail_state.offset()) as u16),
        Constraint::Fill(1),
    ])
    .areas(area);

    let widths = [
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(MAX_DATE_LENGTH as u16),
    ];

    // mailboxes
    if mailbox_area.height > 0 {
        let mailbox_rows: Vec<Row<'_>> = match &mailbox_column.mailboxes {
            Loadable::NotLoaded => {
                vec![Row::new([Cell::from("Mailboxes not requested yet.")
                    .style(Style::new().fg(scheme.primary.into_color()))
                    .column_span(widths.len() as u16)])]
            }
            Loadable::Loading => {
                let throbber = Throbber::default()
                    .throbber_style(Style::new().fg(scheme.primary.into_color()))
                    .to_symbol_span(&state.throbber);

                let line = Cell::from(Line::from(vec![
                    throbber,
                    Span::styled(
                        "Fetching mailboxes...",
                        Style::new().fg(scheme.primary.into_color()),
                    ),
                ]))
                .column_span(widths.len() as u16);

                vec![Row::new([line])]
            }
            Loadable::Loaded(mailboxes) => mailboxes
                .iter()
                .map(|mailbox| {
                    let unread_style = if mailbox.unread_mails > 0 {
                        Style::new().fg(scheme.primary_container.into_color())
                    } else {
                        Style::new().fg(scheme.secondary.into_color())
                    };

                    Row::new([
                        Cell::new(FOLDER_ICON).style(unread_style),
                        Cell::new(mailbox.name.as_str()),
                        Cell::new(format!("{}", mailbox.unread_mails)).style(unread_style),
                    ])
                })
                .collect(),

            Loadable::Error(_) => {
                vec![Row::new([Cell::new("Couldn't fetch mailboxes")
                    .column_span(widths.len() as u16)
                    .style(Style::new().fg(scheme.error.into_color()))])]
            }
        };

        frame.render_stateful_widget(
            Table::new(mailbox_rows, widths).row_highlight_style(
                Style::new()
                    .bg(scheme.primary_container.into_color())
                    .fg(scheme.on_primary_container.into_color()),
            ),
            mailbox_area,
            &mut mailbox_column.mailbox_state,
        );
    }

    // mails
    if mails_area.height > 0 {
        let mail_rows: Vec<Row<'_>> = match &mailbox_column.mails {
            Loadable::NotLoaded => {
                vec![Row::new([Cell::from("Mails not requested yet.")
                    .style(Style::new().fg(scheme.primary.into_color()))
                    .column_span(widths.len() as u16)])]
            }
            Loadable::Loading => {
                let throbber = Throbber::default()
                    .throbber_style(Style::new().fg(scheme.primary.into_color()))
                    .to_symbol_span(&state.throbber);

                let line = Cell::from(Line::from(vec![
                    throbber,
                    Span::styled(
                        "Fetching mails...",
                        Style::new().fg(scheme.primary.into_color()),
                    ),
                ]))
                .column_span(widths.len() as u16);

                vec![Row::new([line])]
            }
            Loadable::Loaded(mails) => mails
                .iter()
                .map(|mail| match mail {
                    Loadable::NotLoaded => Row::new([
                        Cell::from("Mail not requested yet.").column_span(widths.len() as u16)
                    ])
                    .style(Style::new().fg(scheme.primary.into_color())),
                    Loadable::Loading => {
                        let throbber = Throbber::default()
                            .throbber_style(Style::new().fg(scheme.primary.into_color()))
                            .to_symbol_span(&state.throbber);

                        let line = Cell::from(Line::from(vec![
                            throbber,
                            Span::styled(
                                "Fetching mail...",
                                Style::new().fg(scheme.primary.into_color()),
                            ),
                        ]))
                        .column_span(widths.len() as u16);

                        Row::new([line])
                    }
                    Loadable::Loaded(data) => {
                        let unread_symbol = if !data.keywords.contains(&MailKeyword::Seen) {
                            MAIL_UNREAD_ICON
                        } else {
                            PLACEHOLDER
                        };

                        let subject = data
                            .subject
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("<No subject>");

                        let received_at = data.received_at.format("%b %e, %Y").to_string();

                        Row::new([
                            Cell::from(unread_symbol)
                                .style(Style::new().fg(scheme.primary_container.into_color())),
                            Cell::from(subject).style(Style::new().fg(scheme.primary.into_color())),
                            Cell::from(received_at)
                                .style(Style::new().fg(scheme.on_tertiary_container.into_color())),
                        ])
                    }
                    Loadable::Error(_) => Row::new([Cell::from("Couldn't fetch mail.")
                        .column_span(widths.len() as u16)
                        .style(Style::new().fg(scheme.error.into_color()))]),
                })
                .collect(),
            Loadable::Error(_) => {
                vec![Row::new([Cell::new("Couldn't fetch mails")
                    .column_span(widths.len() as u16)
                    .style(Style::new().fg(scheme.error.into_color()))])]
            }
        };

        frame.render_stateful_widget(
            Table::new(mail_rows, widths).row_highlight_style(
                Style::new()
                    .bg(scheme.primary_container.into_color())
                    .fg(scheme.on_primary_container.into_color()),
            ),
            mails_area,
            &mut mailbox_column.mail_state,
        );
    }
}

fn render_thread_column(
    scheme: &Scheme,
    thread_id: ThreadId,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let key = state.get_account_ctx().as_key(thread_id);
    match state.thread_columns.get_mut(&key).unwrap() {
        Loadable::NotLoaded => unreachable!(),
        Loadable::Loading => {
            frame.render_stateful_widget(
                Throbber::default().label("Loading thread"),
                area,
                &mut state.throbber,
            );
        }
        Loadable::Error(err) => {
            frame.render_widget(
                Paragraph::new(err.as_str()).style(Style::new().fg(scheme.error.into_color())),
                area,
            );
        }
        Loadable::Loaded(column) => {
            let widths = [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(MAX_DATE_LENGTH as u16),
            ];

            let rows: Vec<Row<'_>> = {
                let mut rows = Vec::with_capacity(column.len());

                for mail in &column.mails {
                    let unread_symbol = if !mail.keywords.contains(&MailKeyword::Seen) {
                        MAIL_UNREAD_ICON
                    } else {
                        PLACEHOLDER
                    };

                    let subject = mail
                        .subject
                        .as_ref()
                        .map(|s| {
                            if mail.has_attachment {
                                format!("{} {}", PAPERCLIP, s)
                            } else {
                                s.clone()
                            }
                        })
                        .unwrap_or("<No subject>".to_string());

                    let received_at = mail.received_at.format("%b %e, %Y").to_string();

                    rows.push(Row::new([
                        Cell::from(unread_symbol)
                            .style(Style::new().fg(scheme.primary_container.into_color())),
                        Cell::from(subject).style(Style::new().fg(scheme.primary.into_color())),
                        Cell::from(received_at)
                            .style(Style::new().fg(scheme.on_tertiary_container.into_color())),
                    ]));
                }

                rows
            };

            frame.render_stateful_widget(
                Table::new(rows, widths).row_highlight_style(
                    Style::new()
                        .bg(scheme.primary_container.into_color())
                        .fg(scheme.on_primary_container.into_color()),
                ),
                area,
                &mut column.state,
            );
        }
    }
}

fn render_mail_preview(
    scheme: &Scheme,
    mail_id: MailId,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let key = state.get_account_ctx().as_key(mail_id);
    match state.mail_previews.get(&key).expect("State is there") {
        Loadable::NotLoaded => {
            frame.render_widget(
                Paragraph::new("Preview not loaded yet.")
                    .style(Style::new().fg(scheme.tertiary.into_color())),
                area,
            );
        }
        Loadable::Loading => {
            frame.render_stateful_widget(
                Throbber::default()
                    .label("Loading preview...")
                    .throbber_style(Style::new().fg(scheme.primary.into_color())),
                area,
                &mut state.throbber,
            );
        }
        Loadable::Error(err) => {
            frame.render_widget(
                Paragraph::new(format!("Couldn't load mail preview:\n{err}"))
                    .style(Style::new().fg(scheme.error.into_color())),
                area,
            );
        }
        Loadable::Loaded(mail) => {
            let received_at = mail.received_at.format("%c").to_string();

            // headers
            let header_widths = [
                Constraint::Length("Received at:".len() as u16),
                Constraint::Fill(1),
            ];

            let headers_rows = {
                let mut rows = Vec::with_capacity(5);

                let header_style = Style::new().fg(scheme.primary.into_color()).bold();
                let value_style = Style::new().fg(scheme.secondary.into_color());

                if let Some(from) = &mail.from {
                    rows.push(Row::new(vec![
                        Cell::from("From:").style(header_style),
                        Cell::from(from.to_string()).style(value_style),
                    ]))
                }

                if let Some(to) = &mail.to {
                    rows.push(Row::new(vec![
                        Cell::from("To:").style(header_style),
                        Cell::from(to.to_string()).style(value_style),
                    ]));
                }

                if let Some(cc) = &mail.cc {
                    rows.push(Row::new(vec![
                        Cell::from("Cc:").style(header_style),
                        Cell::from(cc.to_string()).style(value_style),
                    ]));
                }

                if let Some(subject) = &mail.subject {
                    rows.push(Row::new(vec![
                        Cell::from("Subject:").style(header_style),
                        Cell::from(subject.as_str()).style(value_style),
                    ]));
                }

                rows.push(Row::new(vec![
                    Cell::from("Received at:").style(header_style),
                    Cell::from(received_at.as_str()).style(value_style),
                ]));

                rows
            };

            let [headers_area, rest] = {
                Layout::vertical([
                    Constraint::Length(headers_rows.len() as u16 + 1),
                    Constraint::Fill(1),
                ])
                .spacing(1)
                .areas(area)
            };

            frame.render_widget(
                Table::new(headers_rows, header_widths).block(
                    Block::new()
                        .title(" Headers ")
                        .title_alignment(HorizontalAlignment::Center)
                        .borders(Borders::TOP)
                        .style(Style::new().fg(scheme.outline.into_color())),
                ),
                headers_area,
            );

            // preview
            let rest_area = if let Some(preview_content) = &mail.preview
                && !preview_content.trim().is_empty()
            {
                let [preview_area, rest] = Layout::vertical([
                    Constraint::Length(preview_content.lines().count() as u16 + 1),
                    Constraint::Fill(1),
                ])
                .spacing(1)
                .areas(rest);

                frame.render_widget(
                    Paragraph::new(preview_content.as_str())
                        .style(Style::new().fg(scheme.secondary.into_color()))
                        .block(
                            Block::new()
                                .title(" Content (preview) ")
                                .title_alignment(HorizontalAlignment::Center)
                                .borders(Borders::TOP)
                                .style(Style::new().fg(scheme.outline.into_color())),
                        )
                        .wrap(Wrap { trim: false }),
                    preview_area,
                );

                rest
            } else {
                rest
            };

            // attachments
            if let Some(attachments) = &mail.attachments
                && !attachments.is_empty()
            {
                let widths = [
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length("123,1 KB".len() as u16),
                ];

                let rows: Vec<Row<'_>> = attachments
                    .iter()
                    .map(|attachment| {
                        let paperclip = Cell::from(PAPERCLIP)
                            .style(Style::new().fg(scheme.secondary.into_color()));
                        let name = Cell::from(attachment.name.as_str())
                            .style(Style::new().fg(scheme.primary.into_color()));
                        let size =
                            Cell::from(Text::from(format_size(attachment.size)).right_aligned())
                                .style(Style::new().fg(scheme.tertiary.into_color()));

                        Row::new([paperclip, name, size])
                    })
                    .collect();

                frame.render_widget(
                    Table::new(rows, widths).block(
                        Block::new()
                            .title(" Attachments ")
                            .title_alignment(HorizontalAlignment::Center)
                            .borders(Borders::TOP)
                            .style(Style::new().fg(scheme.outline.into_color())),
                    ),
                    rest_area,
                );
            }
        }
    }
}

fn render_left_separation_lines(scheme: &Scheme, frame: &mut Frame, area: Rect) {
    let [left_area, _rest] =
        Layout::horizontal([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    frame.render_widget(
        Fill::new(SEPARATION_LINE).style(Style::new().fg(scheme.outline.into_color())),
        left_area,
    );
}

fn format_size(size: usize) -> String {
    const UNITS: [(usize, &str); 3] = [(1_000_000_000, "GB"), (1_000_000, "MB"), (1_000, "KB")];

    for (unit, suffix) in UNITS {
        if size >= unit {
            let tenths = (size * 10 + unit / 2) / unit;
            if tenths >= 10 {
                return format!("{},{} {suffix}", tenths / 10, tenths % 10);
            }
        }
    }

    format!("{size} B")
}
