mod render_data;

use super::state::ViewVariant;
use crate::{
    mail_viewer::state::ScrollAction,
    utils::ui::{ScreenOverlay, ScreenState, input::Input, palette::Palette},
};
use pulldown_cmark_mdcat::ratatui::{RenderOptions, Renderer};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, palette::material::YELLOW},
    text::Text,
    widgets::{
        Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Tabs, Widget,
    },
};
pub use render_data::RenderData;

#[derive(Default)]
pub struct MailViewer {}

impl StatefulWidget for MailViewer {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(mut data) = state.get_render_data() {
            let [main_panel, view_mode] =
                Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

            render_view_modes(view_mode, buf, &mut data);
            render_mail_content(main_panel, buf, &mut data);
            // render_attachment_list(bottom, buf, &mut data);
        } else {
            // loading screen
        }

        render_overlay(area, buf, state);
    }
}

fn render_view_modes(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    Widget::render(
        Tabs::new(["Text", "Markdown (HTML)", "Attachments"])
            .block(Block::bordered().title("Tabs"))
            .highlight_style(Style::new().fg(YELLOW.c500))
            .select(Some(data.variant as usize)),
        area,
        buf,
    );
}

/// Rendering implementations
// TODO: Respect the area size before scrolling.
//    If the whole mail can be fitted within the area rect, there's no need to add the scrollbars.
fn render_mail_content(area: Rect, buf: &mut Buffer, data: &mut RenderData) {
    match data.variant {
        ViewVariant::Markdown => {
            let Some(html) = data.mail.rest.html_body.clone() else {
                Widget::render(
                    Paragraph::new("This mail has no html body.").block(Block::bordered()),
                    area,
                    buf,
                );

                return;
            };
            let markdown = html_to_markdown_rs::convert(&html, None).unwrap();
            let content = markdown.content.unwrap();

            let renderer = Renderer::new(RenderOptions::default().width(area.width));
            let text = renderer.text_from_str(&content).unwrap();

            adjust_scrollbars(
                &text,
                area,
                data.vertical,
                data.horizontal,
                data.scroll_queue,
            );

            Widget::render(
                Paragraph::new(text).block(Block::bordered()).scroll((
                    data.vertical.get_position() as u16,
                    data.horizontal.get_position() as u16,
                )),
                area,
                buf,
            );
        }
        ViewVariant::Text => {
            let Some(content) = data.mail.rest.text_body.clone() else {
                Widget::render(
                    Paragraph::new("This mail has no plain text body.").block(Block::bordered()),
                    area,
                    buf,
                );
                return;
            };

            let text = Text::from(content);

            adjust_scrollbars(
                &text,
                area,
                data.vertical,
                data.horizontal,
                data.scroll_queue,
            );

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
                data.vertical,
            );

            StatefulWidget::render(
                Scrollbar::new(ScrollbarOrientation::HorizontalBottom),
                area,
                buf,
                data.horizontal,
            );
        }

        ViewVariant::Attachments => {
            todo!()
        }
    }
}

fn render_attachment_list(_area: Rect, _buf: &mut Buffer, _data: &mut RenderData) {
    // if data.mail.has_attachment {
    //     let attachments = &data.mail.rest.attachments;

    //     let builder = ListBuilder::new(|context| {
    //         const HEIGHT: u16 = 1;

    //         let attachment = &attachments[context.index];
    //         let widget = AttachmentWidget {
    //             name: attachment.name.as_deref().unwrap_or("<unnamed>"),
    //             ty: attachment.content_type.as_deref().unwrap_or("<unknown>"),
    //         };

    //         (widget, HEIGHT)
    //     });

    //     let list =
    //         ListView::new(builder, attachments.len()).block(Block::bordered().title("Attachments"));

    //     StatefulWidget::render(list, area, buf, &mut tui_widget_list::ListState::default());
    // }
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

fn adjust_scrollbars(
    text: &Text,
    area: Rect,
    vertical: &mut ScrollbarState,
    horizontal: &mut ScrollbarState,
    queue: &mut Option<ScrollAction>,
) {
    let amount_unseen_lines = text.height().saturating_sub(area.height as usize);
    let amount_unseen_columns = text.width().saturating_sub(area.width as usize);

    if let Some(action) = queue.take() {
        match action {
            ScrollAction::ScrollUp(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position(pos.saturating_sub(amount));
            }
            ScrollAction::ScrollDown(amount) => {
                let pos = vertical.get_position();
                *vertical = vertical.position((pos + amount).min(text.height()));
            }
            ScrollAction::ScrollHalfPageDown => {
                let prev_pos = vertical.get_position();
                let new_pos = prev_pos + area.height as usize / 2;
                *vertical = vertical.position(new_pos.min(amount_unseen_lines));
            }
            ScrollAction::ScrollHalfPageUp => {
                let prev_pos = vertical.get_position();
                *vertical = vertical.position(prev_pos.saturating_sub(area.height as usize / 2));
            }
            ScrollAction::SetTop => vertical.first(),
            ScrollAction::SetBottom => vertical.last(),
            ScrollAction::ScrollHalfPageRight => {
                let prev_pos = horizontal.get_position();
                let new_pos = prev_pos + area.width as usize / 2;
                *horizontal = horizontal.position(new_pos.min(amount_unseen_columns));
            }

            ScrollAction::ScrollHalfPageLeft => {
                let prev_pos = horizontal.get_position();
                *horizontal = horizontal.position(prev_pos.saturating_sub(area.width as usize / 2));
            }
        }
    }

    // restrict height
    *vertical = vertical.content_length(amount_unseen_lines);

    // restrict width
    *horizontal = horizontal.content_length(amount_unseen_columns);
}
