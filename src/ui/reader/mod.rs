mod attachments_tab;
mod message;
mod message_request;
mod user_action;
mod view;

use crate::{
    config::Username,
    types::{AccountId, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId},
    ui::{
        Layer,
        reader::attachments_tab::AttachmentsTab,
        utils::keybindmanager::{HandleEvent, KeybindManager},
    },
};
use crossterm::event::Event;
use ratatui::{
    layout::Size,
    widgets::{Paragraph, ScrollbarState, Wrap},
};
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::Message;
pub use message_request::MessageRequest;
pub use view::view;

pub struct State {
    keybindings: KeybindManager<UserAction>,
    throbber: ThrobberState,

    selected_tab: SelectedTab,
    selected_body_type: SelectedBodyType,

    ctx: Option<Ctx>,
    headers: Option<color_eyre::Result<ReaderHeaders>>,
    text_body: Option<color_eyre::Result<Option<TextBody>>>,
    html_body: Option<color_eyre::Result<Option<HtmlBody>>>,
    attachments: Option<AttachmentsTab>,

    mailbox_body_area_size: Option<Size>,
}

impl State {
    pub fn new() -> Self {
        Self {
            throbber: ThrobberState::default(),
            keybindings: KeybindManager::new(HashMap::from([
                ("q", UserAction::Quit),
                ("<Tab>", UserAction::FocusNextTab),
                ("h", UserAction::Back),
                (":", UserAction::OpenCommandPalette),
                ("j", UserAction::NavigateDown),
                ("k", UserAction::NavigateUp),
                ("gg", UserAction::NavigateToTop),
                ("ge", UserAction::NavigateToBottom),
            ])),

            selected_tab: SelectedTab::Mail,
            selected_body_type: SelectedBodyType::Html,

            ctx: None,
            headers: None,
            text_body: None,
            html_body: None,
            attachments: None,

            mailbox_body_area_size: None,
        }
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        self.throbber.calc_next();

        match msg {
            Message::Event(event) => self.handle_event(event),
            Message::UserAction(action) => self.handle_user_action(action),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
            Message::Reset {
                username,
                account_id,
                mail_id,
            } => self.handle_reset(username, account_id, mail_id),

            Message::SetHeaders(headers) => self.handle_set_headres(headers),
            Message::SetTextBody(body) => self.handle_set_text_body(body),
            Message::SetHtmlBody(body) => self.handle_set_html_body(body),
            Message::SetAttachments(attachments) => self.handle_set_attachments(attachments),

            Message::SetMailBodySize(size) => self.handle_set_body_size(size),
        }
    }
}

// message handlers
impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Paste(_) => vec![],
            Event::Resize(width, height) => {
                self.handle_set_body_size(Size::from((width, height)));
                vec![]
            }
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                HandleEvent::Action(action) => self.handle_user_action(action),
                HandleEvent::Registered => vec![],
                HandleEvent::Cancel => vec![],
            },
        }
    }

    fn handle_user_action(&mut self, action: UserAction) -> Vec<super::Message> {
        debug!("{:?}", action);

        match action {
            UserAction::OpenCommandPalette => self.open_command_palette(),
            UserAction::OpenTextBody => self.open_text_body(),
            UserAction::OpenHtmlBody => self.open_html_body(),
            UserAction::NavigateDown => self.navigate_down(),
            UserAction::NavigateUp => self.navigate_up(),
            UserAction::NavigateToTop => self.navigate_to_top(),
            UserAction::NavigateToBottom => self.navigate_to_bottom(),
            UserAction::NavigateHalfPageDown => self.navigate_half_page_down(),
            UserAction::NavigateHalfPageUp => self.navigate_half_page_up(),
            UserAction::NavigateRight => todo!(),
            UserAction::NavigateLeft => todo!(),
            UserAction::FocusNextTab => self.focus_next_tab(),
            UserAction::Quit => self.quit(),
            UserAction::Back => self.back(),
        }
    }

    fn handle_selected_palette_entry(&mut self, entry: String) -> Vec<super::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        vec![super::Message::Reader(Message::UserAction(action))]
    }

    fn handle_reset(
        &mut self,
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    ) -> Vec<super::Message> {
        self.selected_tab = SelectedTab::Mail;
        self.selected_body_type = SelectedBodyType::Html;

        self.ctx = Some(Ctx {
            username,
            account_id,
            mail_id,
        });
        self.headers = None;
        self.text_body = None;
        self.html_body = None;
        self.attachments = None;
        vec![]
    }

    fn handle_set_headres(
        &mut self,
        headers: color_eyre::Result<ReaderHeaders>,
    ) -> Vec<super::Message> {
        self.headers = Some(headers);
        vec![]
    }

    fn handle_set_text_body(
        &mut self,
        body: color_eyre::Result<MailDataTextBody>,
    ) -> Vec<super::Message> {
        self.text_body = Some(body.map(|body| body.content.map(TextBody::new)));
        vec![]
    }

    fn handle_set_html_body(
        &mut self,
        body: color_eyre::Result<MailDataHtmlBody>,
    ) -> Vec<super::Message> {
        self.html_body = Some(body.map(HtmlBody::new));
        vec![]
    }

    fn handle_set_attachments(
        &mut self,
        attachments: color_eyre::Result<Vec<MailDataAttachment>>,
    ) -> Vec<super::Message> {
        self.attachments = Some(AttachmentsTab::new(attachments));
        vec![]
    }

    fn handle_set_body_size(&mut self, size: Size) -> Vec<super::Message> {
        if self
            .mailbox_body_area_size
            .is_some_and(|current_size| current_size == size)
        {
            return vec![];
        }

        self.mailbox_body_area_size = Some(size);

        if let Some(Ok(Some(text_body))) = self.text_body.as_mut() {
            text_body.update_scrollbar_to_new_size(size);
        }

        if let Some(Ok(Some(html_body))) = self.html_body.as_mut() {
            html_body.update_scrollbar_to_new_height(size);
        }

        vec![]
    }
}

// user-action handlers
impl State {
    fn open_command_palette(&self) -> Vec<super::Message> {
        let entries = UserAction::palette_options();

        vec![super::Message::OpenPalette {
            entries,
            map: |entry| super::Message::Reader(Message::SelectedPaletteEntry(entry)),
        }]
    }

    fn open_text_body(&mut self) -> Vec<super::Message> {
        self.selected_body_type = SelectedBodyType::Text;
        let ctx = self.ctx.as_ref().unwrap();

        match self.text_body {
            Some(_) => vec![],
            None => vec![
                MessageRequest::GetTextBody {
                    username: ctx.username.clone(),
                    account_id: ctx.account_id.clone(),
                    mail_id: ctx.mail_id.clone(),
                }
                .into(),
            ],
        }
    }

    fn open_html_body(&mut self) -> Vec<super::Message> {
        self.selected_body_type = SelectedBodyType::Html;
        let ctx = self.ctx.as_ref().unwrap();

        match self.html_body {
            Some(_) => vec![],
            None => vec![
                MessageRequest::GetHtmlBody {
                    username: ctx.username.clone(),
                    account_id: ctx.account_id.clone(),
                    mail_id: ctx.mail_id.clone(),
                }
                .into(),
            ],
        }
    }

    fn navigate_down(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Mail => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(Some(text_body))) = &mut self.text_body {
                        text_body.navigate_down();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(Some(html_body))) = &mut self.html_body {
                        html_body.navigate_down();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = &mut self.attachments {
                    tab.navigate_down();
                }
            }
        };
        vec![]
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Mail => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(Some(text_body))) = &mut self.text_body {
                        text_body.navigate_up();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(Some(html_body))) = &mut self.html_body {
                        html_body.navigate_up();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = &mut self.attachments {
                    tab.navigate_up();
                }
            }
        };
        vec![]
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Mail => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(Some(text_body))) = &mut self.text_body {
                        text_body.navigate_to_top();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(Some(html_body))) = &mut self.html_body {
                        html_body.navigate_to_top();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = &mut self.attachments {
                    tab.navigate_to_top();
                }
            }
        };

        vec![]
    }

    fn navigate_half_page_down(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Mail) {
            return vec![];
        }

        let Some(body_size) = self.mailbox_body_area_size else {
            return vec![];
        };

        let offset = (body_size.height / 2) as usize;

        match self.selected_body_type {
            SelectedBodyType::Text => {
                if let Some(Ok(Some(text_body))) = self.text_body.as_mut() {
                    let content_height = text_body.content_height(body_size);
                    let current_position = text_body.scrollbar.get_position();
                    text_body.scrollbar = text_body
                        .scrollbar
                        .position((current_position + offset).min(content_height));
                }
            }
            SelectedBodyType::Html => {
                if let Some(Ok(Some(html_body))) = self.html_body.as_mut() {
                    let content_height = html_body.content_height(body_size).unwrap();
                    let current_position = html_body.scrollbar.get_position();

                    html_body.scrollbar = html_body
                        .scrollbar
                        .position((current_position + offset).min(content_height));
                }
            }
        }

        vec![]
    }

    fn navigate_half_page_up(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Mail) {
            return vec![];
        }

        let Some(body_size) = self.mailbox_body_area_size else {
            return vec![];
        };

        let offset = (body_size.height / 2) as usize;

        match self.selected_body_type {
            SelectedBodyType::Text => {
                if let Some(Ok(Some(text_body))) = self.text_body.as_mut() {
                    let current_position = text_body.scrollbar.get_position();

                    text_body.scrollbar = text_body
                        .scrollbar
                        .position(current_position.saturating_sub(offset));
                }
            }
            SelectedBodyType::Html => {
                if let Some(Ok(Some(html_body))) = self.html_body.as_mut() {
                    let current_position = html_body.scrollbar.get_position();

                    html_body.scrollbar = html_body
                        .scrollbar
                        .position(current_position.saturating_sub(offset));
                }
            }
        }

        vec![]
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Mail => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(Some(text_body))) = &mut self.text_body {
                        text_body.navigate_to_bottom();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(Some(html_body))) = &mut self.html_body {
                        html_body.navigate_to_bottom();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = &mut self.attachments {
                    tab.navigate_to_bottom();
                }
            }
        };
        vec![]
    }

    fn focus_next_tab(&mut self) -> Vec<super::Message> {
        self.selected_tab = match self.selected_tab {
            SelectedTab::Mail => SelectedTab::Attachments,
            SelectedTab::Attachments => SelectedTab::Mail,
        };
        vec![]
    }

    fn quit(&self) -> Vec<super::Message> {
        vec![super::Message::Quit]
    }

    fn back(&self) -> Vec<super::Message> {
        vec![super::Message::Back]
    }
}

enum SelectedTab {
    Mail,
    Attachments,
}

enum SelectedBodyType {
    Text,
    Html,
}

pub struct ReaderHeaders {
    pub from: Option<String>,
    pub to: Option<String>,
    pub cc: Option<String>,
    pub subject: Option<String>,
    pub received_at: String,
}

impl ReaderHeaders {
    const MAX_AMOUNT_HEADERS: usize = 5;
    const LONGEST_HEADER_LENGTH: usize = "Received at:".len();
}

struct TextBody {
    pub content: String,
    pub scrollbar: ScrollbarState,
}

impl TextBody {
    fn new(content: String) -> Self {
        let scrollbar = ScrollbarState::new(content.lines().count());

        Self { content, scrollbar }
    }

    fn update_scrollbar_to_new_size(&mut self, new_size: Size) {
        let content_height = self.content_height(new_size);
        self.scrollbar = self.scrollbar.content_length(content_height);
    }

    fn content_height(&self, render_area_size: Size) -> usize {
        let amount_lines = Paragraph::new(self.content.as_str())
            .wrap(Wrap { trim: false })
            .line_count(render_area_size.width);

        // for whatever reason there seems to be a off-by-one-error in the calculation.
        // So just to be sure: Assume that the height is less than one
        let body_area_height = render_area_size.height.saturating_sub(1);

        amount_lines.saturating_sub(body_area_height as usize)
    }
}
struct HtmlBody {
    html: String,
    markdown: std::io::Result<String>,
    scrollbar: ScrollbarState,
}

impl HtmlBody {
    pub fn new(html: MailDataHtmlBody) -> Option<Self> {
        let content = html.content?;
        let markdown = htmd::convert(content.as_str());

        let scrollbar = {
            let amount_lines = markdown
                .as_ref()
                .ok()
                .map(|content| content.lines().count())
                .unwrap_or(0);
            ScrollbarState::new(amount_lines)
        };

        Some(Self {
            html: content,
            markdown,
            scrollbar,
        })
    }

    fn update_scrollbar_to_new_height(&mut self, new_size: Size) {
        let Some(content_height) = self.content_height(new_size) else {
            return;
        };

        self.scrollbar = self.scrollbar.content_length(content_height);
    }

    fn content_height(&self, render_area_size: Size) -> Option<usize> {
        let raw_markdown = self.markdown.as_ref().ok()?;

        let markdown_text = pulldown_cmark_mdcat::ratatui::text_from_str(
            raw_markdown.as_str(),
            // `body_area` is created by split up the bigger area into `Fill` and `Length(1)`.
            // `+ 1` should be the `Length(1)` for the `scrollbar_area`
            render_area_size.width + 1,
        )
        .ok()?;

        let amount_lines = Paragraph::new(markdown_text)
            .wrap(Wrap { trim: false })
            .line_count(render_area_size.width);

        Some(amount_lines.saturating_sub(render_area_size.height as usize))
    }
}

struct Ctx {
    username: Username,
    account_id: AccountId,
    mail_id: MailId,
}

trait Scrollable {
    fn scrollbar(&mut self) -> &mut ScrollbarState;

    fn navigate_down(&mut self) {
        self.scrollbar().next();
    }

    fn navigate_up(&mut self) {
        self.scrollbar().prev();
    }

    fn navigate_to_top(&mut self) {
        self.scrollbar().first();
    }

    fn navigate_to_bottom(&mut self) {
        self.scrollbar().last();
    }
}

impl Scrollable for TextBody {
    fn scrollbar(&mut self) -> &mut ScrollbarState {
        &mut self.scrollbar
    }
}

impl Scrollable for HtmlBody {
    fn scrollbar(&mut self) -> &mut ScrollbarState {
        &mut self.scrollbar
    }
}
