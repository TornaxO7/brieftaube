use pulldown_cmark_mdcat::ratatui::{RenderOptions, Renderer};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};
use throbber_widgets_tui::Throbber;

use crate::backend::{MailDataHtmlBody, types::Loadable};

pub struct MarkdownReader<'a> {
    pub html_body: &'a Loadable<MailDataHtmlBody>,
}

impl<'a> StatefulWidget for MarkdownReader<'a> {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        match self.html_body {
            Loadable::NotRequested => {
                Widget::render(Paragraph::new("Not requested yet"), area, buf);
            }
            Loadable::Requested { .. } => {
                StatefulWidget::render(
                    Throbber::default()
                        .label("Fetching html body part of mail...")
                        .throbber_set(throbber_widgets_tui::BRAILLE_SIX),
                    area,
                    buf,
                    &mut model.throbber,
                );
            }
            Loadable::Loaded(html) => {
                let content = htmd::convert(html.0.as_str()).unwrap();

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
    }
}
