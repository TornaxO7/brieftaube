use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};

pub struct TextReader<'a> {
    pub text_body: Option<&'a str>,
}

impl<'a> StatefulWidget for TextReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let Some(content) = self.text_body else {
            Widget::render(
                Paragraph::new("Fetching text body part of mail...").block(Block::bordered()),
                area,
                buf,
            );
            return;
        };

        let text = Text::from(content);

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
