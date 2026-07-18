use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Style,
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table, Widget},
};
use std::sync::{
    Arc,
    atomic::{AtomicU16, Ordering},
};
use throbber_widgets_tui::{Throbber, ThrobberState};

#[derive(Default)]
pub struct Statusbar {}

impl StatefulWidget for Statusbar {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered();
        let header_area = block.inner(area);

        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .areas(header_area);

        let [_padding1, throbber, screen_name, _padding2] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(state.screen_name.len() as u16),
            Constraint::Fill(2),
        ])
        .areas(center);

        Widget::render(block, area, buf);

        if state.show_counter {
            render_error_warning_info_state(left, buf, &state.counter);
        }
        render_throbber(throbber, buf, &mut state.throbber_state);
        render_screen_name(screen_name, buf, state.screen_name);
        render_keypress(right, buf, &state.keypresses);
    }
}

fn render_error_warning_info_state(area: Rect, buf: &mut Buffer, counter: &super::Counter) {
    fn get_cell(
        label: &str,
        count: &Arc<AtomicU16>,
        active: Style,
        inactive: Style,
    ) -> Cell<'static> {
        let value = count.load(Ordering::Relaxed);
        let style = if value > 0 { active.bold() } else { inactive };

        Cell::new(format!("{}: {}", label, value)).style(style)
    }

    let errors = get_cell(
        "E",
        &counter.errors,
        Style::default().light_red(),
        Style::default().red(),
    );

    let warnings = get_cell(
        "W",
        &counter.warnings,
        Style::default().light_yellow(),
        Style::default().yellow(),
    );

    let infos = get_cell(
        "I",
        &counter.infos,
        Style::default().light_green(),
        Style::default().green(),
    );

    let rows = [Row::new([errors, warnings, infos])];
    let widths = [
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(5),
    ];
    Widget::render(Table::new(rows, widths).column_spacing(2), area, buf);
}

fn render_throbber(area: Rect, buf: &mut Buffer, state: &mut ThrobberState) {
    StatefulWidget::render(
        Throbber::default()
            .style(Style::default().bold().light_blue())
            .throbber_set(throbber_widgets_tui::BRAILLE_SIX_DOUBLE)
            .use_type(throbber_widgets_tui::WhichUse::Spin),
        area,
        buf,
        state,
    );
}

fn render_screen_name(area: Rect, buf: &mut Buffer, name: &str) {
    Widget::render(
        Paragraph::new(name)
            .alignment(HorizontalAlignment::Center)
            .style(Style::default().bold().light_blue()),
        area,
        buf,
    );
}

fn render_keypress(area: Rect, buf: &mut Buffer, keypress: &str) {
    Widget::render(
        Paragraph::new(keypress).alignment(HorizontalAlignment::Right),
        area,
        buf,
    );
}
