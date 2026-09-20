use crate::{
    THEME,
    types::{MailId, ParentMailboxId, ROOT_MAILBOX_ID, ThreadId},
    ui::{
        Loadable,
        mailfs::{ColumnStackEntry, user_column::UserColumnEntry},
        statusbar::Statusbar,
    },
    utils::IntoColor,
};
use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Cell, Fill, Paragraph, Row, Table, Wrap},
};
use throbber_widgets_tui::Throbber;

const MAX_DATE_LENGTH: usize = "Jan 10, 1996".len();

const COLLAPSED: &str = "▸";
const UNCOLLAPSED: &str = "▾";
const UNCOLLAPSED_CHILD: &str = "├";
const UNCOLLAPSED_END: &str = "└";
const SEPARATION_LINE: &str = "│";
const PLACEHOLDER: &str = " ";

const FOLDER_ICON: &str = "🖿";
const MAIL_UNREAD_ICON: &str = "●";

// TODO: create cache for rendering

pub fn view(state: &mut super::State, frame: &mut Frame, area: Rect) {
    let theme = THEME.get().unwrap();
    let scheme = &theme.schemes.dark;

    let [path_area, columns_area, statusbar_area] = Layout::vertical([
        Constraint::Length(1),
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

    let Some(UserColumnEntry::Account(account)) = state.users_column.get_selected_entry() else {
        return;
    };

    path.push(Span::styled(
        account.name.as_str(),
        Style::new()
            .fg(scheme.on_primary.into_color())
            .bg(scheme.primary.into_color()),
    ));

    frame.render_widget(Line::from(path), area);
}

fn render_statusbar(_scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let layer_name = {
        let column_type = match state.column_stack.last().unwrap() {
            ColumnStackEntry::Users => "Users",
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
    let [left_area, sep1, middle_area, sep2, right_area] = Layout::horizontal([
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
            let Some(selected_entry) = state.users_column.get_selected_entry() else {
                return;
            };

            match selected_entry {
                UserColumnEntry::User(_user_ctx) => {
                    // IDEA: Maybe render some stats from the config?
                }
                UserColumnEntry::Account(_account_data) => {
                    render_mailbox_column(scheme, ROOT_MAILBOX_ID, state, frame, area);
                }
                UserColumnEntry::AccountNotLoaded => {
                    frame.render_widget(
                        Paragraph::new("Accounts haven't been loaded yet.")
                            .style(Style::new().fg(scheme.primary.into_color())),
                        area,
                    );
                }
                UserColumnEntry::AccountLoading => {
                    frame.render_widget(
                        Paragraph::new("Connecting to server...")
                            .style(Style::new().fg(scheme.primary.into_color())),
                        area,
                    );
                }
                UserColumnEntry::AccountError(error) => {
                    frame.render_widget(
                        Paragraph::new(format!("Couldn't connect to server:\n{error}"))
                            .wrap(Wrap { trim: false })
                            .style(Style::new().fg(scheme.error.into_color())),
                        area,
                    );
                }
            }
        }
        ColumnStackEntry::Mailbox(mailbox_id) => {
            // TODO: get selected entry of this given mailbox and render it.
            // render_mailbox_column(scheme, mailbox_id.clone(), state, frame, area)
        }
        ColumnStackEntry::Thread(thread_id) => {
            todo!();
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
        ColumnStackEntry::Users => render_user_accounts_column(scheme, state, frame, area),
        ColumnStackEntry::Mailbox(mailbox_id) => {
            render_mailbox_column(scheme, mailbox_id, state, frame, area)
        }
        ColumnStackEntry::Thread(thread_id) => {
            render_thread_column(scheme, thread_id, state, frame, area)
        }
    }
}

fn render_user_accounts_column(
    scheme: &Scheme,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let rows: Vec<Row> = {
        let mut rows = vec![];

        for user in state.users_column.users.iter() {
            if user.is_collapsed {
                rows.push(Row::new([
                    Cell::from(COLLAPSED),
                    Cell::from(user.config.username.as_str())
                        .style(Style::new().fg(scheme.secondary.into_color())),
                ]));
                continue;
            } else {
                rows.push(Row::new([
                    Cell::from(UNCOLLAPSED),
                    Cell::from(user.config.username.as_str())
                        .style(Style::new().fg(scheme.secondary.into_color())),
                ]));
            }

            // accounts
            match &user.accounts {
                Loadable::NotLoaded => {
                    rows.push(Row::new([
                        Cell::from(UNCOLLAPSED_END),
                        Cell::from("Not loaded"),
                    ]));
                }
                Loadable::Loading => {
                    let throbber = Throbber::default()
                        .throbber_style(Style::default().fg(scheme.primary.into_color()))
                        .to_symbol_span(&state.throbber);

                    let line = Cell::from(Line::from(vec![
                        throbber,
                        Span::styled(
                            "Loading account...",
                            Style::default().fg(scheme.primary.into_color()),
                        ),
                    ]));

                    rows.push(Row::new([Cell::from(UNCOLLAPSED_END), line]))
                }
                Loadable::Loaded(accounts) => {
                    let (last, rest) = accounts.split_last().unwrap();

                    for account in rest {
                        rows.push(Row::new([
                            Cell::from(UNCOLLAPSED_CHILD),
                            Cell::from(account.name.as_str()),
                        ]));
                    }

                    rows.push(Row::new([
                        Cell::from(UNCOLLAPSED_END),
                        Cell::from(last.name.as_str()),
                    ]));
                }
                Loadable::Error(_) => rows.push(Row::new([
                    Cell::from(UNCOLLAPSED_END),
                    Cell::from("Error: Login failed")
                        .style(Style::default().fg(scheme.error.into_color())),
                ])),
            }
        }

        rows
    };

    let widths = [Constraint::Length(2), Constraint::Fill(1)];

    frame.render_stateful_widget(
        Table::new(rows, widths).row_highlight_style(
            Style::new()
                .fg(scheme.on_primary_container.into_color())
                .bg(scheme.primary_container.into_color()),
        ),
        area,
        &mut state.users_column.state,
    );
}

fn render_mailbox_column(
    scheme: &Scheme,
    mailbox_id: ParentMailboxId,
    state: &mut super::State,
    frame: &mut Frame,
    area: Rect,
) {
    let Some(account) = state.users_column.get_selected_account() else {
        return;
    };

    let key = account.as_key(mailbox_id);
    let mailbox_column = state.mailbox_columns.get_mut(&key).unwrap();

    let widths = [
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(MAX_DATE_LENGTH as u16),
    ];

    let rows: Vec<Row<'_>> = {
        let mut rows = Vec::with_capacity(area.height.into());

        // mailboxes
        match &mailbox_column.mailboxes {
            Loadable::NotLoaded => {
                rows.push(Row::new([Cell::from("Mailboxes not requested yet.")
                    .style(scheme.primary.into_color())
                    .column_span(widths.len() as u16)]));
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

                rows.push(Row::new([line]))
            }
            Loadable::Loaded(mailboxes) => {
                for mailbox in mailboxes {
                    let unread_style = if mailbox.unread_mails > 0 {
                        Style::new().fg(scheme.primary_container.into_color())
                    } else {
                        Style::new().fg(scheme.secondary.into_color())
                    };

                    rows.push(Row::new([
                        Cell::new(FOLDER_ICON).style(unread_style),
                        Cell::new(mailbox.name.as_str()),
                        Cell::new(format!("{}", mailbox.unread_mails)).style(unread_style),
                    ]))
                }
            }
            Loadable::Error(_) => {
                rows.push(Row::new([Cell::new("Couldn't fetch mailboxes")
                    .column_span(widths.len() as u16)
                    .style(Style::new().fg(scheme.error.into_color()))]));
            }
        }

        // mails

        rows
    };

    frame.render_stateful_widget(
        Table::new(rows, widths).row_highlight_style(
            Style::new()
                .bg(scheme.primary_container.into_color())
                .fg(scheme.on_primary_container.into_color()),
        ),
        area,
        &mut mailbox_column.state,
    );
}

fn render_thread_column(
    _scheme: &Scheme,
    _thread_id: ThreadId,
    _state: &mut super::State,
    _frame: &mut Frame,
    _area: Rect,
) {
}

fn render_mail_preview(
    _scheme: &Scheme,
    _mail_id: MailId,
    _state: &mut super::State,
    _frame: &mut Frame,
    _area: Rect,
) {
}

fn render_left_separation_lines(scheme: &Scheme, frame: &mut Frame, area: Rect) {
    let [left_area, _rest] =
        Layout::horizontal([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    frame.render_widget(
        Fill::new(SEPARATION_LINE).style(Style::new().fg(scheme.outline.into_color())),
        left_area,
    );
}
