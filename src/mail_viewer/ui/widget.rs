use crate::utils::ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget, Widget},
};
use tui_widget_list::{ListBuilder, ListView};

#[derive(Default)]
pub struct MailViewer {}

impl StatefulWidget for MailViewer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [top, bottom] = if state.mail.has_attachment() {
            Layout::vertical([Constraint::Percentage(80), Constraint::Fill(0)]).areas(area)
        } else {
            [area, Rect::default()]
        };

        render_mail_content(top, buf, state);
        render_attachment_list(bottom, buf, state);

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
}

/// Rendering implementations
// TODO: Respect the area size before scrolling.
//    If the whole mail can be fitted within the area rect, there's no need to add the scrollbars.
fn render_mail_content(area: Rect, buf: &mut Buffer, state: &mut super::State) {
    Widget::render(
        Paragraph::new(state.mail_str.as_str())
            .block(Block::bordered())
            .scroll((
                state.vertical.get_position() as u16,
                state.horizontal.get_position() as u16,
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
        &mut state.vertical,
    );

    StatefulWidget::render(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
        area,
        buf,
        &mut state.horizontal,
    );
}

fn render_attachment_list(area: Rect, buf: &mut Buffer, state: &super::State) {
    if let Some(attachments) = state.mail.attachments() {
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
