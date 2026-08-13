use ratatui::widgets::ScrollbarState;

#[derive(Default)]
pub struct TextViewer {
    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
}
