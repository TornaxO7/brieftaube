use crate::{THEME, utils::IntoColor};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Text,
};

pub enum StatusMsgType {
    Error,
    Info,
}

pub struct StatusMsg<'a> {
    pub msg: &'a str,
    pub ty: StatusMsgType,
}

pub fn draw(
    layer_name: &str,
    status_msg: StatusMsg<'_>,
    pressed_keys: &str,
    frame: &mut Frame,
    area: Rect,
) {
    let layer_name = format!(" {layer_name} ");
    let pressed_keys = format!(" {pressed_keys} ");

    let [left, center, right] = Layout::horizontal([
        Constraint::Length(layer_name.len() as u16),
        Constraint::Fill(1),
        Constraint::Length(pressed_keys.len() as u16),
    ])
    .areas(area);

    let theme = THEME.get().unwrap();
    let scheme = &theme.schemes.dark;

    frame.render_widget(
        Text::raw(layer_name).style(
            Style::new()
                .fg(scheme.on_primary_container.into_color())
                .bg(scheme.primary_container.into_color()),
        ),
        left,
    );

    let status_style = match status_msg.ty {
        StatusMsgType::Error => Style::new()
            .fg(scheme.on_error_container.into_color())
            .bg(scheme.error_container.into_color()),
        StatusMsgType::Info => Style::new()
            .fg(scheme.on_secondary_container.into_color())
            .bg(scheme.secondary_container.into_color()),
    };

    let msg = format!(" {} ", status_msg.msg);
    frame.render_widget(Text::raw(msg).style(status_style), center);

    frame.render_widget(
        Text::raw(pressed_keys).style(
            Style::new()
                .fg(scheme.on_tertiary_container.into_color())
                .bg(scheme.tertiary_container.into_color()),
        ),
        right,
    );
}
