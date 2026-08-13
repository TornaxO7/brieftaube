use ratatui::widgets::ScrollbarState;

#[derive(Default)]
pub struct MarkdownViewer {
    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
}
