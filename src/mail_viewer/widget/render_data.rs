use ratatui::widgets::ScrollbarState;

use crate::mail_viewer::{
    state::{ScrollAction, ViewVariant},
    types::FullMailDisplay,
};

pub struct RenderData<'a> {
    pub variant: ViewVariant,
    pub mail: FullMailDisplay,
    pub vertical: &'a mut ScrollbarState,
    pub horizontal: &'a mut ScrollbarState,
    pub scroll_queue: &'a mut Option<ScrollAction>,
}
