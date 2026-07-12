mod list;

use crate::ui::{ScreenOverlay, ScreenState, utils};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};

const MAIL_LIST_PANEL_TITLE: &str = "Mails";
const PREVIEW_PANEL_TITLE: &str = "Mail content";

#[derive(Default)]
pub struct RootMails {}

impl StatefulWidget for RootMails {
    type State = super::State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [headerbar, content] = area.layout(&Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(0),
        ]));

        let [mail_list, preview] = content.layout(&Layout::horizontal([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]));

        self.render_mail_list(mail_list, buf, state);
        self.render_preview(preview, buf, state);
        self.render_headerbar(headerbar, buf, state);

        if let Some(state) = state.overlay() {
            let a = area.centered(Constraint::Percentage(80), Constraint::Percentage(85));
            Widget::render(Clear, a, buf);
            match state {
                ScreenOverlay::Palette(state) => {
                    StatefulWidget::render(utils::palette::Palette::new(), a, buf, state);
                }
                ScreenOverlay::Input(state) => {
                    StatefulWidget::render(utils::input::Input::default(), a, buf, state)
                }
            }
        }
    }
}

/// Render functions
impl RootMails {
    fn render_mail_list(&self, area: Rect, buf: &mut Buffer, state: &mut super::State) {
        match state.root_mails.as_ref() {
            Some(root_mails) => {
                StatefulWidget::render(
                    list::MailListWidget::new(root_mails).block(
                        Block::bordered()
                            .title(MAIL_LIST_PANEL_TITLE)
                            .title_alignment(HorizontalAlignment::Center),
                    ),
                    area,
                    buf,
                    &mut state.list_state,
                );
            }
            None => Widget::render(
                Paragraph::new("Loading...").block(Block::bordered().title(MAIL_LIST_PANEL_TITLE)),
                area,
                buf,
            ),
        }
    }

    fn render_preview(&self, area: Rect, buf: &mut Buffer, state: &mut super::State) {
        if let (Some(root_mails), Some(idx)) = (&state.root_mails, state.list_state.selected) {
            let mail = &root_mails[idx];
            let preview = mail.preview().unwrap();

            Widget::render(
                Paragraph::new(preview).block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
                area,
                buf,
            )
        } else {
            Widget::render(
                Paragraph::new("No mail selected")
                    .block(Block::bordered().title(PREVIEW_PANEL_TITLE)),
                area,
                buf,
            )
        }
    }

    fn render_headerbar(&self, area: Rect, buf: &mut Buffer, _state: &mut super::State) {
        let block = Block::bordered();
        let header_area = block.inner(area);

        let [left, center, right] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Fill(0),
            Constraint::Fill(0),
        ])
        .areas(header_area);

        Widget::render(block, area, buf);
        Widget::render(
            Paragraph::new("Left").alignment(HorizontalAlignment::Left),
            left,
            buf,
        );
        Widget::render(
            Paragraph::new("Center").alignment(HorizontalAlignment::Center),
            center,
            buf,
        );
        Widget::render(
            Paragraph::new("Right").alignment(HorizontalAlignment::Right),
            right,
            buf,
        );
    }
}
