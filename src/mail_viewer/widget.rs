use crate::mail_viewer::{
    model::{
        Viewer, attachments::AttachmentsViewer, markdown::MarkdownViewer, metadata::MetadataViewer,
        text::TextViewer,
    },
    types::MailDisplay,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, palette::material::YELLOW},
    widgets::{Block, StatefulWidget, Tabs, Widget},
};

pub struct MailViewer;

impl StatefulWidget for MailViewer {
    type State = super::Model;

    fn render(self, area: Rect, buf: &mut Buffer, model: &mut Self::State) {
        let [main_panel, tab_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

        render_tabs(tab_area, buf, model.viewer);
        render_viewer(main_panel, buf, model);
    }
}

fn render_tabs(area: Rect, buf: &mut Buffer, viewer: Viewer) {
    let idx = match viewer {
        Viewer::Metadata => 0,
        Viewer::Text => 1,
        Viewer::Markdown => 2,
        Viewer::Attachments => 3,
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
    let mail = MailDisplay::from(model.get_mail());

    match model.viewer {
        Viewer::Metadata => StatefulWidget::render(
            MetadataViewer { mail: &mail },
            area,
            buf,
            &mut model.metadata,
        ),

        Viewer::Text => StatefulWidget::render(
            TextViewer {
                text_body: mail.text_body.as_ref().map(|body| body.0.as_str()),
            },
            area,
            buf,
            &mut model.text,
        ),
        Viewer::Markdown => StatefulWidget::render(
            MarkdownViewer {
                html_body: mail.html_body.as_ref().map(|body| body.0.as_str()),
            },
            area,
            buf,
            &mut model.markdown,
        ),
        Viewer::Attachments => StatefulWidget::render(
            AttachmentsViewer {
                attachments: mail.attachments.unwrap().as_slice(),
            },
            area,
            buf,
            &mut model.attachments,
        ),
    }
}
