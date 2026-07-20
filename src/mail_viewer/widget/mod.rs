mod render_data;

use crate::utils::ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};
pub use render_data::RenderData;
use tui_widget_list::{ListBuilder, ListView};

#[derive(Default)]
pub struct MailViewer {}

impl StatefulWidget for MailViewer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut data) = state.get_render_data() {
            let [top, bottom] = if data.mail.has_attachment {
                Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area)
            } else {
                [area, Rect::default()]
            };

            render_mail_content(top, buf, &mut data);
            render_attachment_list(bottom, buf, &mut data);
        } else {
            // loading screen
        }

        render_overlay(area, buf, state);
    }
}

/// Rendering implementations
// TODO: Respect the area size before scrolling.
//    If the whole mail can be fitted within the area rect, there's no need to add the scrollbars.
fn render_mail_content(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    let text = data.mail.rest.text_body.clone().unwrap();

    {
        let amount_unseen_lines = text.lines().count().saturating_sub(area.height as usize);
        *data.vertical = data.vertical.content_length(amount_unseen_lines);
    }

    {
        let amount_unseen_columns = text
            .lines()
            .next()
            .map(|line| line.len().saturating_sub(area.width as usize))
            .unwrap_or(0);

        *data.horizontal = data.horizontal.content_length(amount_unseen_columns);
    }

    Widget::render(
        Paragraph::new(text).block(Block::bordered()).scroll((
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
        &mut data.vertical,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
        area,
        buf,
        &mut data.horizontal,
    );
}

fn render_attachment_list(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    // if data.mail.has_attachment {
    //  if let Some(attachments) = data.mail.rest.attachments {
    //     let builder = ListBuilder::new(|context| {
    //         const HEIGHT: u16 = 1;

    //         let attachment = &attachments[context.index];

    //         let widget = AttachmentWidget {
    //             name: attachment.name().unwrap(),
    //             ty: attachment.content_type().unwrap(),
    //         };

    //         (widget, HEIGHT)
    //     });

    //     let list =
    //         ListView::new(builder, attachments.len()).block(Block::bordered().title("Attachments"));

    //     StatefulWidget::render(list, area, buf, &mut tui_widget_list::ListState::default());
    // }
    // }
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

fn render_overlay(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    if let Some(state) = state.overlay() {
        let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
        Widget::render(Clear, a, buf);
        match state {
            ScreenOverlay::Palette(state) => {
                StatefulWidget::render(Palette::new(), a, buf, state);
            }
            ScreenOverlay::Input(state) => StatefulWidget::render(Input::new(), a, buf, state),
        }
    }
}
