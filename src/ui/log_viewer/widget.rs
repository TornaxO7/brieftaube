use crate::ui::{ScreenOverlay, ScreenState, utils};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Clear, StatefulWidget, Widget},
};
use tui_logger::TuiLoggerWidget;

#[derive(Default)]
pub struct LogViewer {}

impl StatefulWidget for LogViewer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TuiLoggerWidget::default()
            .block(Block::bordered().title(state.log_file_path()))
            .style_error(Style::default().red())
            .style_warn(Style::default().yellow())
            .style_info(Style::default().green())
            .output_file(false)
            .output_line(false)
            .output_target(false)
            .output_timestamp(Some("[%H:%M:%S]".to_string()))
            .state(state.scroll_state())
            .render(area, buf);

        if let Some(state) = state.overlay() {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Widget::render(Clear, a, buf);
            match state {
                ScreenOverlay::Palette(state) => {
                    StatefulWidget::render(utils::palette::Palette::new(), a, buf, state);
                }
                ScreenOverlay::Input(state) => {
                    StatefulWidget::render(utils::input::Input::default(), a, buf, state)
                }
            }
        }
    }
}
