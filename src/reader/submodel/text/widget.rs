use crate::backend::{MailDataTextBody, types::Loadable};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};
use throbber_widgets_tui::Throbber;

pub struct TextReader<'a> {
    pub text_body: &'a mut Option<Loadable<MailDataTextBody>>,
}

impl<'a> StatefulWidget for TextReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        match self.text_body {
            None => {
                Widget::render(Paragraph::new("Not requested yet."), area, buf);
            }
            Some(Loadable::Loading(state)) => {
                StatefulWidget::render(
                    Throbber::default()
                        .label("Fetching text body part of mail...")
                        .throbber_set(throbber_widgets_tui::BRAILLE_SIX),
                    area,
                    buf,
                    state,
                );
            }
            Some(Loadable::Loaded(text)) => {
                let text = Text::from(text.0.as_str());

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
    }
}
