use crate::{
    THEME,
    ui::{Loadable, mailfs::user_column::UserColumnEntry, statusbar::Statusbar},
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

const COLLAPSED: &str = "▸";
const UNCOLLAPSED: &str = "▾";
const UNCOLLAPSED_CHILD: &str = "├";
const UNCOLLAPSED_END: &str = "└";
const SEPARATION_LINE: &str = "│";
const PLACEHOLDER: &str = " ";

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

    // account name
    let Some(idx) = state.users_column.state.selected() else {
        return;
    };

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

fn render_statusbar(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let layer_name = format!("Mailifs({})", state.mode);

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
    if state.column_stack.len() <= 1 {
        return;
    }

    if state.column_stack.len() == 2 {
        render_user_accounts_column(scheme, state, frame, area);
    }
}

fn render_middle_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    if state.column_stack.len() == 1 {
        render_user_accounts_column(scheme, state, frame, area);
    } else {
    }
}

fn render_right_column(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let middle_is_user_accounts = state.column_stack.len() == 1;
    if middle_is_user_accounts {
        let Some(selected_entry) = state.users_column.get_selected_entry() else {
            return;
        };

        match selected_entry {
            UserColumnEntry::User(_user_ctx) => {}
            UserColumnEntry::Account(_account_data) => {
                todo!()
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
        return;
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

fn render_mail_list_column(state: &mut super::State, frame: &mut Frame, area: Rect) {}
fn render_mail_preview(state: &mut super::State, frame: &mut Frame, area: Rect) {}

fn render_left_separation_lines(scheme: &Scheme, frame: &mut Frame, area: Rect) {
    let [left_area, _rest] =
        Layout::horizontal([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

    frame.render_widget(
        Fill::new(SEPARATION_LINE).style(Style::new().fg(scheme.outline.into_color())),
        left_area,
    );
}
