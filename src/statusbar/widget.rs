use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, Widget},
};

#[derive(Default)]
pub struct Statusbar {}

impl StatefulWidget for Statusbar {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered();
        let header_area = block.inner(area);

        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Fill(0),
            Constraint::Fill(0),
        ])
        .areas(header_area);

        Widget::render(block, area, buf);

        render_error_warning_info_state(left, buf, state);
        render_screen_name(center, buf, state.screen_name);

        Widget::render(
            Paragraph::new("Right").alignment(HorizontalAlignment::Right),
            right,
            buf,
        );
    }
}

fn render_error_warning_info_state(area: Rect, buf: &mut Buffer, state: &super::State) {
    let errors = Cell::new(format!("E: {}", state.errors)).style(state.error_style);
    let warnings = Cell::new(format!("W: {}", state.warnings)).style(state.warning_style);
    let info = Cell::new(format!("I: {}", state.info)).style(state.info_style);

    let rows = [Row::new([errors, warnings, info])];
    let widths = [
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
    ];
    Widget::render(Table::new(rows, widths).column_spacing(2), area, buf);
}

fn render_screen_name(area: Rect, buf: &mut Buffer, name: &str) {
    Widget::render(
        Paragraph::new(name).alignment(HorizontalAlignment::Center),
        area,
        buf,
    );
}
