use crate::utils::IntoColor;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use material_theme_loader::Scheme;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{StatefulWidget, Widget},
};
use throbber_widgets_tui::{Throbber, ThrobberState};

#[derive(Debug, Clone, Copy)]
pub enum StatusMsgType {
    Error,
    Info,
    Loading,
}

pub struct StatusMsg {
    pub msg: String,
    pub ty: StatusMsgType,
}

#[derive(Default)]
pub struct StatusbarState {
    status_msg: Option<StatusMsg>,
    pressed_keys: String,
    throbber: ThrobberState,
}

impl StatusbarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_message(&mut self, msg: String, ty: StatusMsgType) {
        self.status_msg = Some(StatusMsg { msg, ty });
    }

    pub fn clear_message(&mut self) {
        self.status_msg = None;
    }

    pub fn reset_pressed_keys(&mut self) {
        self.pressed_keys.clear();
    }

    pub fn register_key_event(&mut self, event: KeyEvent) {
        let c = match event.code {
            KeyCode::Char(c) => c,
            KeyCode::Tab => {
                self.pressed_keys.push_str("<Tab>");
                return;
            }
            KeyCode::Backspace => {
                self.pressed_keys.push_str("<BS>");
                return;
            }
            KeyCode::Esc => {
                self.pressed_keys.push_str("<Esc>");
                return;
            }
            KeyCode::BackTab => {
                self.pressed_keys.push_str("<BTab>");
                return;
            }
            _ => {
                self.pressed_keys.push('?');
                return;
            }
        };

        if event.modifiers.contains(KeyModifiers::CONTROL) {
            self.pressed_keys.push_str(&format!("<C-{c}>"));
        } else if event.modifiers.contains(KeyModifiers::ALT) {
            self.pressed_keys.push_str(&format!("<A-{c}>"));
        } else if event.modifiers.contains(KeyModifiers::SHIFT) {
            self.pressed_keys.push_str(&format!("<S-{c}>"));
        } else {
            self.pressed_keys.push(c);
        }
    }
}

pub struct StatusbarWidget<'a> {
    layer_name: Option<&'a str>,
    scheme: &'a Scheme,
}

impl<'a> StatusbarWidget<'a> {
    pub fn new(scheme: &'a Scheme) -> Self {
        Self {
            scheme,
            layer_name: None,
        }
    }

    pub fn layer_name(mut self, layer_name: &'a str) -> Self {
        self.layer_name = Some(layer_name);
        self
    }
}

impl<'a> StatefulWidget for StatusbarWidget<'a> {
    type State = StatusbarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        let layer_name = self
            .layer_name
            .map(|layer_name| format!(" {layer_name} "))
            .unwrap_or_default();

        let pressed_keys = format!(" {} ", state.pressed_keys.as_str());

        let [left_area, center_area, right_area] = Layout::horizontal([
            Constraint::Length(layer_name.len() as u16),
            Constraint::Fill(1),
            Constraint::Length(pressed_keys.len() as u16),
        ])
        .areas(area);

        Widget::render(
            Text::from(layer_name).style(
                Style::new()
                    .fg(self.scheme.on_primary_container.into_color())
                    .bg(self.scheme.primary_container.into_color()),
            ),
            left_area,
            buf,
        );

        Widget::render(
            state
                .status_msg
                .as_ref()
                .map(|status_msg| {
                    let status_style = match status_msg.ty {
                        StatusMsgType::Error => Style::new()
                            .fg(self.scheme.on_error_container.into_color())
                            .bg(self.scheme.error_container.into_color()),
                        StatusMsgType::Info | StatusMsgType::Loading => Style::new()
                            .fg(self.scheme.on_secondary_container.into_color())
                            .bg(self.scheme.secondary_container.into_color()),
                    };

                    let text = match status_msg.ty {
                        StatusMsgType::Error | StatusMsgType::Info => {
                            Text::from(format!(" {} ", status_msg.msg))
                        }
                        StatusMsgType::Loading => {
                            state.throbber.calc_next();
                            let throbber = Throbber::default().to_symbol_span(&state.throbber);
                            Text::from(Line::from(
                                [
                                    Span::from(" "),
                                    throbber,
                                    Span::from(status_msg.msg.as_str()),
                                ]
                                .as_slice(),
                            ))
                        }
                    };

                    text.style(status_style)
                })
                .unwrap_or(
                    Text::from("").style(
                        Style::new()
                            .fg(self.scheme.on_secondary_container.into_color())
                            .bg(self.scheme.secondary_container.into_color()),
                    ),
                ),
            center_area,
            buf,
        );

        Widget::render(
            Text::from(pressed_keys)
                .style(
                    Style::new()
                        .fg(self.scheme.on_primary_container.into_color())
                        .bg(self.scheme.primary_container.into_color()),
                )
                .right_aligned(),
            right_area,
            buf,
        );
    }
}
