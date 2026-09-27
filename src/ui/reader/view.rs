use material_theme_loader::Scheme;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

pub fn view(scheme: &Scheme, state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [tabs_area, content_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
}
