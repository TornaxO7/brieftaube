use crate::ui::{ScreenOverlay, ScreenState, utils};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct Composer {}

impl StatefulWidget for Composer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mail = state.get_mail();

        let [top, _bottom] =
            Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area);

        render_mail_content(mail, top, buf);
        // render_attachment_list(mail, bottom, buf);

        if let Some(state) = state.overlay() {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Widget::render(Clear, a, buf);

            match state {
                ScreenOverlay::Palette(state) => {
                    StatefulWidget::render(utils::palette::Palette::new(), a, buf, state);
                }
                ScreenOverlay::Input(state) => {
                    StatefulWidget::render(utils::input::Input::new(), a, buf, state)
                }
            }
        }
    }
}

fn render_mail_content(mail: &str, area: Rect, buf: &mut Buffer) {
    tracing::debug!("{:#?}", mail);
    Widget::render(Paragraph::new(mail).block(Block::bordered()), area, buf)
}
