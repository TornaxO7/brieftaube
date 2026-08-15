use ratatui::widgets::StatefulWidget;

pub struct MarkdownViewer;

impl StatefulWidget for MarkdownViewer {
    type State = super::Model;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        todo!()
    }
}
