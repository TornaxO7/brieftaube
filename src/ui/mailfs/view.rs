use crate::ui::statusbar::{self, Statusbar};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

pub fn view(state: &mut super::State, frame: &mut Frame, area: Rect) {
    let [path_area, columns_area, statusbar_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);

    frame.render_widget(
        Statusbar::default()
            .layer_name("Mailfs(normal)")
            .status_msg(statusbar::StatusMsg {
                msg: "hello",
                ty: statusbar::StatusMsgType::Info,
            }),
        statusbar_area,
    );
}
