use crate::{ui::mailfs::columns::AccountsColumn, utils::IntoColor};
use material_theme_loader::Scheme;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Cell, Row, StatefulWidget, Table},
};

pub struct AccountsColumnWidget<'a> {
    pub scheme: &'a Scheme,
    pub render_line: bool,
}

impl<'a> StatefulWidget for AccountsColumnWidget<'a> {
    type State = AccountsColumn;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let rows: Vec<Row<'_>> = {
            let mut rows = Vec::with_capacity(state.accounts.len());

            for account in &state.accounts {
                let row = Row::new([Cell::new(account.name.as_str())
                    .style(Style::new().fg(self.scheme.primary.into_color()))]);

                rows.push(row);
            }

            rows
        };

        let widths = [Constraint::Fill(1)];

        StatefulWidget::render(
            Table::new(rows, widths).row_highlight_style(
                Style::new()
                    .bg(self.scheme.primary_container.into_color())
                    .fg(self.scheme.on_primary_container.into_color()),
            ),
            area,
            buf,
            &mut state.state,
        );
    }
}
