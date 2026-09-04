use crate::{THEME, utils::IntoColor};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Text,
    widgets::Widget,
};

pub enum StatusMsgType {
    Error,
    Info,
}

pub struct StatusMsg<'a> {
    pub msg: &'a str,
    pub ty: StatusMsgType,
}

#[derive(Default)]
pub struct Statusbar<'a> {
    layer_name: Option<&'a str>,
    status_msg: Option<StatusMsg<'a>>,
    pressed_keys: Option<&'a str>,
}

impl<'a> Statusbar<'a> {
    pub fn layer_name(mut self, layer_name: &'a str) -> Self {
        self.layer_name = Some(layer_name);
        self
    }

    pub fn status_msg(mut self, status_msg: StatusMsg<'a>) -> Self {
        self.status_msg = Some(status_msg);
        self
    }

    pub fn pressed_keys(mut self, pressed_keys: &'a str) -> Self {
        self.pressed_keys = Some(pressed_keys);
        self
    }
}

impl<'a> Widget for Statusbar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layer_name = self
            .layer_name
            .map(|layer_name| format!(" {layer_name} "))
            .unwrap_or_default();

        let pressed_keys = self
            .pressed_keys
            .map(|pressed_keys| format!(" {pressed_keys} "))
            .unwrap_or_default();

        let [left, center, right] = Layout::horizontal([
            Constraint::Length(layer_name.len() as u16),
            Constraint::Fill(1),
            Constraint::Length(pressed_keys.len() as u16),
        ])
        .areas(area);

        let theme = THEME.get().unwrap();
        let scheme = &theme.schemes.dark;

        Widget::render(
            Text::raw(layer_name).style(
                Style::new()
                    .fg(scheme.on_primary_container.into_color())
                    .bg(scheme.primary_container.into_color()),
            ),
            left,
            buf,
        );

        let status_msg = self
            .status_msg
            .map(|status_msg| {
                let status_style = match status_msg.ty {
                    StatusMsgType::Error => Style::new()
                        .fg(scheme.on_error_container.into_color())
                        .bg(scheme.error_container.into_color()),
                    StatusMsgType::Info => Style::new()
                        .fg(scheme.on_secondary_container.into_color())
                        .bg(scheme.secondary_container.into_color()),
                };
                let msg = format!(" {} ", status_msg.msg);

                Text::raw(msg).style(status_style)
            })
            .unwrap_or_default();

        Widget::render(status_msg, center, buf);

        Widget::render(
            Text::raw(pressed_keys).style(
                Style::new()
                    .fg(scheme.on_tertiary_container.into_color())
                    .bg(scheme.tertiary_container.into_color()),
            ),
            right,
            buf,
        );
    }
}
