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
    layout::{Rect, Size},
    widgets::ScrollbarState,
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

    size: Size,
}

impl State {
    pub fn new(init_rect: Rect) -> Self {
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

            size: init_rect.as_size(),
        }
    }

    fn get_available_mail_body_area_height(&self) -> u16 {
        const AMOUNT_SEPARATOR_LINES: u16 = 3;

        let mut occupied_height = AMOUNT_SEPARATOR_LINES;

        if let Some(Ok(headers)) = &self.headers {
            occupied_height += headers.amount_entries();
        }

        self.size.height.saturating_sub(occupied_height)
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
        }
    }
}

// message handlers
impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Paste(_) => vec![],
            Event::Resize(width, height) => {
                self.size.width = width;
                self.size.height = height;

                // update scrollbars
                if let Some(Ok(Some(text_body))) = self.text_body.as_mut() {
                    text_body.scrollbar =
                        text_body
                            .content_height()
                            .checked_sub(height)
                            .map(|amount_unseen_lines| match text_body.scrollbar.as_mut() {
                                Some(scrollbar) => {
                                    scrollbar.content_length(amount_unseen_lines as usize)
                                }
                                None => ScrollbarState::new(amount_unseen_lines as usize),
                            });
                }

                if let Some(Ok(Some(html_body))) = self.html_body.as_mut() {
                    html_body.scrollbar = html_body
                        .get_content_height()
                        .and_then(|content_height| content_height.checked_sub(height))
                        .map(|amount_unseen_lines| match html_body.scrollbar.as_mut() {
                            Some(scrollbar) => {
                                scrollbar.content_length(amount_unseen_lines as usize)
                            }
                            None => ScrollbarState::new(amount_unseen_lines as usize),
                        });
                }

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
        let available_body_area_height = self.get_available_mail_body_area_height();
        self.text_body = Some(body.map(|body| {
            body.content
                .map(|content| TextBody::new(content, available_body_area_height))
        }));
        vec![]
    }

    fn handle_set_html_body(
        &mut self,
        body: color_eyre::Result<MailDataHtmlBody>,
    ) -> Vec<super::Message> {
        let available_body_area_height = self.get_available_mail_body_area_height();
        self.html_body = Some(body.map(|body| HtmlBody::new(body, available_body_area_height)));
        vec![]
    }

    fn handle_set_attachments(
        &mut self,
        attachments: color_eyre::Result<Vec<MailDataAttachment>>,
    ) -> Vec<super::Message> {
        self.attachments = Some(AttachmentsTab::new(attachments));
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

    pub fn amount_entries(&self) -> u16 {
        let mut counter = 1; // due to `received_at`

        if self.from.is_some() {
            counter += 1;
        }

        if self.to.is_some() {
            counter += 1;
        }

        if self.cc.is_some() {
            counter += 1;
        }

        if self.subject.is_some() {
            counter += 1;
        }

        counter
    }
}

struct TextBody {
    pub content: String,
    pub scrollbar: Option<ScrollbarState>,
}

impl TextBody {
    fn new(content: String, available_body_area_height: u16) -> Self {
        let mut body = Self {
            content,
            scrollbar: None,
        };

        if let Some(amount_unseen_lines) = body
            .content_height()
            .checked_sub(available_body_area_height)
        {
            body.scrollbar = Some(ScrollbarState::new(amount_unseen_lines as usize));
        }

        body
    }

    fn content_height(&self) -> u16 {
        self.content.lines().count() as u16
    }
}

struct HtmlBody {
    html: String,
    markdown: std::io::Result<String>,

    scrollbar: Option<ScrollbarState>,
}

impl HtmlBody {
    pub fn new(html: MailDataHtmlBody, available_area_height: u16) -> Option<Self> {
        let content = html.content?;
        let markdown = htmd::convert(content.as_str());

        std::fs::write("/tmp/test.md", markdown.as_ref().unwrap()).unwrap();

        let mut body = Self {
            html: content,
            markdown,
            scrollbar: None,
        };

        if let Some(displayed_height) = body.get_content_height() {
            if let Some(amount_unseen_lines) = displayed_height.checked_sub(available_area_height) {
                body.scrollbar = Some(ScrollbarState::new(amount_unseen_lines as usize));
            }
        }

        Some(body)
    }

    fn get_content_height(&self) -> Option<u16> {
        self.markdown
            .as_ref()
            .ok()
            .map(|content| content.lines().count() as u16)
    }
}

struct Ctx {
    username: Username,
    account_id: AccountId,
    mail_id: MailId,
}

trait Scrollable {
    fn scrollbar(&mut self) -> Option<&mut ScrollbarState>;

    fn navigate_down(&mut self) {
        if let Some(scrollbar) = self.scrollbar() {
            scrollbar.next();
        }
    }

    fn navigate_up(&mut self) {
        if let Some(scrollbar) = self.scrollbar() {
            scrollbar.prev();
        }
    }

    fn navigate_to_top(&mut self) {
        if let Some(scrollbar) = self.scrollbar() {
            scrollbar.first();
        }
    }

    fn navigate_to_bottom(&mut self) {
        if let Some(scrollbar) = self.scrollbar() {
            scrollbar.last();
        }
    }
}

impl Scrollable for TextBody {
    fn scrollbar(&mut self) -> Option<&mut ScrollbarState> {
        self.scrollbar.as_mut()
    }
}

impl Scrollable for HtmlBody {
    fn scrollbar(&mut self) -> Option<&mut ScrollbarState> {
        self.scrollbar.as_mut()
    }
}
