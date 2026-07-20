use ratatui::widgets::ScrollbarState;

use crate::mail_viewer::types::FullMailDisplay;

pub struct RenderData<'a> {
    pub horizontal: &'a mut ScrollbarState,
    pub vertical: &'a mut ScrollbarState,
    pub mail: FullMailDisplay,
}
