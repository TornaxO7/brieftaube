mod column_data;
mod render_data;

pub use column_data::ColumnData;
pub use render_data::RenderData;

use crate::utils::ui::ScreenState;

use super::State;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct Mailfs {}

impl StatefulWidget for Mailfs {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(data) = state.render_data() {
            let [left_area, center_area, right_area] = Layout::horizontal([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .areas(area);

            if let Some(left) = &data.left {
                render_column(left_area, buf, left);
            }
            render_line(left_area, buf);
            render_column(center_area, buf, &data.center);
            render_line(center_area, buf);
            if let Some(right) = &data.right {
                render_column(right_area, buf, right);
            }
        } else {
            // loading screen
        }
    }
}

fn render_column(area: Rect, buf: &mut Buffer, data: &ColumnData) {}

fn render_line(area: Rect, buf: &mut Buffer) {
    Block::new().borders(Borders::RIGHT).render(area, buf);
}
