use crate::ui::statusbar::Statusbar;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

pub fn view(state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [path_area, columns_area, statusbar_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(
        Statusbar::default().layer_name("Mailfs(normal)"),
        statusbar_area,
    );
}
