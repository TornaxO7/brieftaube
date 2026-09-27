use crate::{ui::reader::SelectedTab, utils::IntoColor};
use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Tabs},
};

pub fn view(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [tabs_area, content_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);

    render_tab_widgets(scheme, state, frame, tabs_area);
}

fn render_tab_widgets(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    const TAB1: &str = "Content";
    const TAB2: &str = "Attachments";

    let [area, _right_padding] = Layout::horizontal([
        Constraint::Length(2 + TAB1.len() as u16 + 3 + TAB2.len() as u16 + 2),
        Constraint::Fill(1),
    ])
    .areas(area);

    let selected_idx = match state.selected_tab {
        SelectedTab::Mail => 0,
        SelectedTab::Attachments => 1,
    };

    frame.render_widget(
        Tabs::new([TAB1, TAB2])
            .select(selected_idx)
            .block(Block::new().borders(Borders::all() - Borders::BOTTOM))
            .style(Style::new().fg(scheme.outline.into_color()))
            .highlight_style(Style::new().fg(scheme.primary.into_color())),
        area,
    );
}
