use pulldown_cmark_mdcat::ratatui::{RenderOptions, Renderer};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};

pub struct MarkdownReader<'a> {
    pub html_body: Option<&'a str>,
}

impl<'a> StatefulWidget for MarkdownReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let Some(html) = self.html_body else {
            Widget::render(
                Paragraph::new("Fetching html body...").block(Block::bordered()),
                area,
                buf,
            );

            return;
        };
        let markdown = html_to_markdown_rs::convert(&html, None).unwrap();
        let content = markdown.content.unwrap();

        let renderer = Renderer::new(RenderOptions::default().width(area.width));
        let text = renderer.text_from_str(&content).unwrap();

        let (content_area, vertical_scrollbar_area, horizontal_scrollbar_area) =
            crate::reader::submodel::adjust_scrollbars(
                &text,
                area,
                &mut model.vertical,
                &mut model.horizontal,
                model.scroll_action.take(),
            );

        Widget::render(
            Paragraph::new(text).block(Block::bordered()).scroll((
                model.vertical.get_position() as u16,
                model.horizontal.get_position() as u16,
            )),
            content_area,
            buf,
        );

        if let Some(area) = vertical_scrollbar_area {
            StatefulWidget::render(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                buf,
                &mut model.vertical,
            );
        }

        if let Some(area) = horizontal_scrollbar_area {
            StatefulWidget::render(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
                area,
                buf,
                &mut model.horizontal,
            );
        }
    }
}
