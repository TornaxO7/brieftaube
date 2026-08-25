use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};
use tui_logger::TuiLoggerWidget;

pub struct LogViewer;

impl StatefulWidget for LogViewer {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        TuiLoggerWidget::default()
            .block(Block::bordered().title(state.log_file_path()))
            .style_error(Style::default().red())
            .style_warn(Style::default().yellow())
            .style_info(Style::default().green())
            .output_target(false)
            .output_timestamp(Some("[%H:%M:%S]".to_string()))
            .state(state.scroll_state())
            .render(area, buf);
    }
}
