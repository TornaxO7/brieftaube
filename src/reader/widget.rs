use crate::reader::model::{
    Mode, attachments::AttachmentsReader, markdown::MarkdownReader, metadata::MetadataReader,
    text::TextReader,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, palette::material::YELLOW},
    widgets::{Block, StatefulWidget, Tabs, Widget},
};

pub struct Reader;

impl StatefulWidget for Reader {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let [main_panel, tab_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

        render_tabs(tab_area, buf, model.mode);
        render_viewer(main_panel, buf, model);
    }
}

fn render_tabs(area: Rect, buf: &mut Buffer, mode: Mode) {
    let idx = match mode {
        Mode::Metadata => 0,
        Mode::Text => 1,
        Mode::Markdown => 2,
        Mode::Attachments => 3,
    };

    Widget::render(
        Tabs::new(["Metadata", "Text", "Markdown (HTML)", "Attachments"])
            .block(Block::bordered().title("Tabs"))
            .highlight_style(Style::new().fg(YELLOW.c500))
            .select(Some(idx)),
        area,
        buf,
    );
}

/// Rendering implementations
fn render_viewer(area: Rect, buf: &mut Buffer, model: &mut super::Model) {
    let mut mail = model.get_display_mail();

    match model.mode {
        Mode::Metadata => StatefulWidget::render(
            MetadataReader { mail: &mail },
            area,
            buf,
            &mut model.metadata,
        ),

        Mode::Text => StatefulWidget::render(
            TextReader {
                text_body: &mut mail.text_body,
            },
            area,
            buf,
            &mut model.text,
        ),
        Mode::Markdown => StatefulWidget::render(
            MarkdownReader {
                html_body: &mut mail.html_body,
            },
            area,
            buf,
            &mut model.markdown,
        ),
        Mode::Attachments => StatefulWidget::render(
            AttachmentsReader {
                attachments: &mut mail.attachments,
            },
            area,
            buf,
            &mut model.attachments,
        ),
    }
}
