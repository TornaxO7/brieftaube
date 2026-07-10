use jmap_client::email::Email;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{
        Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};
use tui_widget_list::{ListBuilder, ListView};

use crate::ui::{ScreenPalette, palette};

#[derive(Default)]
pub struct MailViewer {}

impl StatefulWidget for MailViewer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut data) = state.get_render_data() {
            let [top, bottom] = if data.mail.has_attachment() {
                Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area)
            } else {
                [area, Rect::default()]
            };

            render_mail_content(&mut data, top, buf);
            render_attachment_list(&data.mail, bottom, buf);
        } else {
            Widget::render(
                Paragraph::new("Loading mail...").block(Block::bordered()),
                area,
                buf,
            );
        }

        if let Some(state) = &mut state.palette() {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Widget::render(Clear, a, buf);
            StatefulWidget::render(palette::Palette::new(), a, buf, state);
        }
    }
}

/// Rendering implementations
// TODO: Respect the area size before scrolling.
//    If the whole mail can be fitted within the area rect, there's no need to add the scrollbars.
fn render_mail_content(data: &mut RenderData, area: Rect, buf: &mut Buffer) {
    Widget::render(
        Paragraph::new(data.mail_str)
            .block(Block::bordered())
            .scroll((
                data.vertical.get_position() as u16,
                data.horizontal.get_position() as u16,
            )),
        area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        }),
        buf,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        buf,
        data.vertical,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
        area,
        buf,
        data.horizontal,
    );
}

fn render_attachment_list(mail: &Email, area: Rect, buf: &mut Buffer) {
    if let Some(attachments) = mail.attachments() {
        let builder = ListBuilder::new(|context| {
            const HEIGHT: u16 = 1;

            let attachment = &attachments[context.index];

            let widget = AttachmentWidget {
                name: attachment.name().unwrap(),
                ty: attachment.content_type().unwrap(),
            };

            (widget, HEIGHT)
        });

        let list =
            ListView::new(builder, attachments.len()).block(Block::bordered().title("Attachments"));

        StatefulWidget::render(list, area, buf, &mut tui_widget_list::ListState::default());
    }
}

#[derive(Debug)]
struct AttachmentWidget<'a> {
    name: &'a str,
    ty: &'a str,
}

impl<'a> Widget for AttachmentWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [left, right] =
            Layout::horizontal([Constraint::Fill(0), Constraint::Fill(0)]).areas(area);

        Widget::render(Paragraph::new(self.name).left_aligned(), left, buf);
        Widget::render(Paragraph::new(self.ty).right_aligned(), right, buf);
    }
}

#[derive(Debug)]
pub struct RenderData<'a> {
    pub mail: &'a Email,
    pub mail_str: &'a str,
    pub vertical: &'a mut ScrollbarState,
    pub horizontal: &'a mut ScrollbarState,
}
