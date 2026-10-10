mod attachments_tab;
mod body;
mod message;
mod message_request;
mod types;
mod user_action;
mod view;

use crate::{
    config::Username,
    types::{
        AccountId, BlobId, CachedAttachment, MailDataHtmlBody, MailDataTextBody, MailId,
        MailProperty,
    },
    ui::{
        EditorContentType, Layer,
        pager::{attachments_tab::AttachmentsTab, message::SaveAttachmentStep},
        statusbar::{StatusMsgType, StatusbarState},
        utils::keybindmanager::{HandleEvent, KeybindManager},
    },
};
use body::*;
use crossterm::event::Event;
use std::{collections::HashMap, str::FromStr};
use throbber_widgets_tui::ThrobberState;
use tracing::debug;
use user_action::UserAction;

pub use message::Message;
pub use message_request::MessageRequest;
pub use types::*;
pub use view::view;

pub struct State {
    keybindings: KeybindManager<UserAction>,
    throbber: ThrobberState,
    statusbar: StatusbarState,
    mode: Mode,

    selected_tab: SelectedTab,
    selected_body_type: SelectedBodyType,

    ctx: Ctx,
    headers: Option<color_eyre::Result<MailHeaders>>,
    text_body: Option<color_eyre::Result<TextBody>>,
    html_body: Option<color_eyre::Result<HtmlBody>>,
    attachments: Option<AttachmentsTab>,
}

impl State {
    pub fn new(username: Username, account_id: AccountId, mail_id: MailId, mode: Mode) -> Self {
        Self {
            mode,
            statusbar: StatusbarState::new(),
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
                ("<C-d>", UserAction::NavigateHalfPageDown),
                ("<C-u>", UserAction::NavigateHalfPageUp),
            ])),

            selected_tab: SelectedTab::Body,
            selected_body_type: SelectedBodyType::Html,

            ctx: Ctx {
                username,
                account_id,
                mail_id,
            },
            headers: None,
            text_body: None,
            html_body: None,
            attachments: None,
        }
    }

    fn is_in_state(&mut self, expected_mode: Option<Mode>, selected_tab: SelectedTab) -> bool {
        let is_in_state = expected_mode.map(|mode| self.mode == mode).unwrap_or(true)
            && self.selected_tab == selected_tab;

        if !is_in_state {
            let msg_suffix = match expected_mode {
                Some(mode) => {
                    format!("in mode `Pager({})` and tab `{}`", mode, selected_tab)
                }
                None => {
                    format!("in tab '{}'", selected_tab)
                }
            };

            let msg = format!("Action can be only applied {}", msg_suffix);

            self.statusbar.set_message(msg, StatusMsgType::Error);
        }

        is_in_state
    }
}

impl Layer<Message> for State {
    fn update(&mut self, msg: Message) -> Vec<super::Message> {
        self.throbber.calc_next();

        match msg {
            Message::Event(event) => self.handle_event(event),
            Message::UserAction(action) => self.handle_user_action(action),
            Message::SelectedPaletteEntry(entry) => self.handle_selected_palette_entry(entry),
            Message::SetHeaders(headers) => self.handle_set_headers(headers),
            Message::SetTextBody(body) => self.handle_set_text_body(body),
            Message::SetHtmlBody(body) => self.handle_set_html_body(body),
            Message::SetAttachments(attachments) => self.handle_set_attachments(attachments),
            Message::SetStatusbarMessage { msg, ty } => self.handle_set_statusbar_message(msg, ty),
            Message::SaveAttachment(step) => self.handle_save_attachment(step),
        }
    }
}

// message handlers
impl State {
    fn handle_event(&mut self, event: Event) -> Vec<super::Message> {
        match event {
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Paste(_) => vec![],
            Event::Resize(_, _) => {
                vec![]
            }
            Event::Key(key_event) => match self.keybindings.handle_event(key_event) {
                HandleEvent::Action(action) => {
                    self.statusbar.reset_pressed_keys();
                    self.handle_user_action(action)
                }
                HandleEvent::Registered => {
                    self.statusbar.register_key_event(key_event);
                    vec![]
                }
                HandleEvent::Cancel => {
                    self.statusbar.reset_pressed_keys();
                    vec![]
                }
            },
        }
    }

    fn handle_user_action(&mut self, action: UserAction) -> Vec<super::Message> {
        debug!("{:?}", action);

        if !matches!(action, UserAction::OpenCommandPalette) {
            self.statusbar.clear_message();
        }

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
            UserAction::FocusNextTab => self.focus_next_tab(),

            UserAction::ReadBodyInEditor => self.read_body_in_editor(),
            UserAction::ReadTextBodyInEditor => self.read_text_body_in_editor(),
            UserAction::ReadMarkdownBodyInEditor => self.read_markdown_body_in_editor(),
            UserAction::ReadHtmlBodyInEditor => self.read_html_body_in_editor(),

            UserAction::DownloadAttachmentAndPasteDestinationPath => {
                self.download_attachment_and_paste_destination_path()
            }

            UserAction::Quit => self.quit(),
            UserAction::Back => self.back(),
        }
    }

    fn handle_selected_palette_entry(&mut self, entry: String) -> Vec<super::Message> {
        let action = UserAction::from_str(entry.as_str()).unwrap();
        vec![super::Message::Pager(Message::UserAction(action))]
    }

    fn handle_set_headers(
        &mut self,
        headers: color_eyre::Result<MailHeaders>,
    ) -> Vec<super::Message> {
        self.headers = Some(headers);
        vec![]
    }

    fn handle_set_text_body(
        &mut self,
        body: color_eyre::Result<MailDataTextBody>,
    ) -> Vec<super::Message> {
        tracing::debug!("Set text body");
        self.text_body = Some(body.map(TextBody::new));
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
        attachments: color_eyre::Result<Vec<CachedAttachment>>,
    ) -> Vec<super::Message> {
        self.attachments = Some(AttachmentsTab::new(attachments));
        vec![]
    }

    fn handle_set_statusbar_message(
        &mut self,
        msg: String,
        ty: StatusMsgType,
    ) -> Vec<super::Message> {
        self.statusbar.set_message(msg, ty);
        vec![]
    }

    fn handle_save_attachment(&mut self, step: SaveAttachmentStep) -> Vec<super::Message> {
        match step {
            SaveAttachmentStep::GetDestinationPath(result) => match result {
                Ok(blob) => {
                    self.statusbar.set_message(
                        "[2/2] Saving attachment".to_string(),
                        StatusMsgType::Loading,
                    );

                    vec![
                        super::Message::OpenPrompt {
                            description: "Destination path".to_string(),
                            map: Box::new(move |path| {
                                Message::SaveAttachment(SaveAttachmentStep::SaveAttachment {
                                    blob: blob.clone(),
                                    path: path.into(),
                                })
                                .into()
                            }),
                        }
                        .into(),
                    ]
                }

                Err(err) => {
                    let msg = format!("Couldn't get attachment: {}", err);
                    self.statusbar.set_message(msg, StatusMsgType::Error);
                    vec![]
                }
            },
            SaveAttachmentStep::SaveAttachment { blob, path } => {
                match std::fs::write(path, blob) {
                    Ok(()) => {
                        self.statusbar
                            .set_message("Attachment saved.".to_string(), StatusMsgType::Info);
                    }
                    Err(err) => {
                        let msg = format!("Couldn't save attachment: {}", err);
                        self.statusbar.set_message(msg, StatusMsgType::Error);
                    }
                };
                vec![]
            }
        }
    }
}

// user-action handlers
impl State {
    fn open_command_palette(&self) -> Vec<super::Message> {
        let entries = UserAction::palette_options();

        vec![super::Message::OpenPalette {
            entries,
            map: |entry| super::Message::Pager(Message::SelectedPaletteEntry(entry)),
        }]
    }

    fn open_text_body(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        self.selected_body_type = SelectedBodyType::Text;

        match &self.text_body {
            Some(_) => vec![],
            None => vec![
                MessageRequest::GetMail {
                    username: self.ctx.username.clone(),
                    account_id: self.ctx.account_id.clone(),
                    mail_id: self.ctx.mail_id.clone(),
                    properties: vec![MailProperty::TextBody],
                    callback: Box::new(move |cached_mail| {
                        vec![Message::SetTextBody(cached_mail.map(MailDataTextBody::from)).into()]
                    }),
                }
                .into(),
            ],
        }
    }

    fn open_html_body(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        self.selected_body_type = SelectedBodyType::Html;

        match self.html_body.as_ref() {
            Some(_) => vec![],
            None => vec![
                MessageRequest::GetMail {
                    username: self.ctx.username.clone(),
                    account_id: self.ctx.account_id.clone(),
                    mail_id: self.ctx.mail_id.clone(),
                    properties: vec![MailProperty::HtmlBody],
                    callback: Box::new(move |cached_mail| {
                        vec![Message::SetHtmlBody(cached_mail.map(MailDataHtmlBody::from)).into()]
                    }),
                }
                .into(),
            ],
        }
    }

    fn navigate_down(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(text_body)) = self.text_body.as_mut() {
                        text_body.navigate_down(1);
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(html_body)) = self.html_body.as_mut() {
                        html_body.navigate_down(1);
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = self.attachments.as_mut() {
                    tab.navigate_down();
                }
            }
        };
        vec![]
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(text_body)) = self.text_body.as_mut() {
                        text_body.navigate_up(1);
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(html_body)) = self.html_body.as_mut() {
                        html_body.navigate_up(1);
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = self.attachments.as_mut() {
                    tab.navigate_up();
                }
            }
        };
        vec![]
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(text_body)) = self.text_body.as_mut() {
                        text_body.navigate_to_top();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(html_body)) = self.html_body.as_mut() {
                        html_body.navigate_to_top();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = self.attachments.as_mut() {
                    tab.navigate_to_top();
                }
            }
        };

        vec![]
    }

    fn navigate_half_page_down(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Body) {
            return vec![];
        }

        match self.selected_body_type {
            SelectedBodyType::Text => {
                if let Some(Ok(text_body)) = self.text_body.as_mut() {
                    text_body.navigate_half_page_down();
                }
            }
            SelectedBodyType::Html => {
                if let Some(Ok(html_body)) = self.html_body.as_mut() {
                    html_body.navigate_half_page_down();
                }
            }
        }

        vec![]
    }

    fn navigate_half_page_up(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Body) {
            return vec![];
        }

        match self.selected_body_type {
            SelectedBodyType::Text => {
                if let Some(Ok(text_body)) = self.text_body.as_mut() {
                    text_body.navigate_half_page_up();
                }
            }
            SelectedBodyType::Html => {
                if let Some(Ok(html_body)) = self.html_body.as_mut() {
                    html_body.navigate_half_page_up();
                }
            }
        }

        vec![]
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    if let Some(Ok(text_body)) = self.text_body.as_mut() {
                        text_body.navigate_to_bottom();
                    }
                }
                SelectedBodyType::Html => {
                    if let Some(Ok(html_body)) = self.html_body.as_mut() {
                        html_body.navigate_to_bottom();
                    }
                }
            },
            SelectedTab::Attachments => {
                if let Some(tab) = self.attachments.as_mut() {
                    tab.navigate_to_bottom();
                }
            }
        };
        vec![]
    }

    fn focus_next_tab(&mut self) -> Vec<super::Message> {
        self.selected_tab = match self.selected_tab {
            SelectedTab::Body => SelectedTab::Attachments,
            SelectedTab::Attachments => SelectedTab::Body,
        };
        tracing::debug!("new tab: {:?}", self.selected_tab);
        vec![]
    }

    fn read_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        match self.selected_body_type {
            SelectedBodyType::Text => self.read_text_body_in_editor(),
            SelectedBodyType::Html => self.read_markdown_body_in_editor(),
        }
    }

    fn read_text_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        let Some(init_text_body) = self.text_body.as_ref() else {
            return vec![
                MessageRequest::GetMail {
                    username: self.ctx.username.clone(),
                    account_id: self.ctx.account_id.clone(),
                    mail_id: self.ctx.mail_id.clone(),
                    properties: vec![MailProperty::TextBody],
                    callback: Box::new(|cached_mail| match cached_mail {
                        Ok(_cache_mail) => {
                            vec![Message::UserAction(UserAction::ReadTextBodyInEditor).into()]
                        }
                        Err(err) => vec![
                            Message::SetStatusbarMessage {
                                msg: format!("Couldn't fetch text body: {}", err),
                                ty: StatusMsgType::Error,
                            }
                            .into(),
                        ],
                    }),
                }
                .into(),
            ];
        };

        if let Ok(text_body) = init_text_body {
            vec![super::Message::OpenInEditor {
                content: text_body.state.content.clone(),
                ty: EditorContentType::Text,
                on_exit: |result| match result {
                    Ok(_) => vec![],
                    Err(err) => {
                        vec![
                            Message::SetStatusbarMessage {
                                msg: err.to_string(),
                                ty: StatusMsgType::Error,
                            }
                            .into(),
                        ]
                    }
                },
            }]
        } else {
            vec![]
        }
    }

    fn read_markdown_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        let Some(init_html_body) = self.html_body.as_ref() else {
            return vec![
                MessageRequest::GetMail {
                    username: self.ctx.username.clone(),
                    account_id: self.ctx.account_id.clone(),
                    mail_id: self.ctx.mail_id.clone(),
                    properties: vec![MailProperty::HtmlBody],
                    callback: Box::new(|cached_mail| match cached_mail {
                        Ok(_cached_mail) => {
                            vec![Message::UserAction(UserAction::ReadMarkdownBodyInEditor).into()]
                        }
                        Err(err) => vec![
                            Message::SetStatusbarMessage {
                                msg: format!("Couldn't fetch html body: {}", err),
                                ty: StatusMsgType::Error,
                            }
                            .into(),
                        ],
                    }),
                }
                .into(),
            ];
        };

        let Ok(html_body) = init_html_body else {
            return vec![];
        };

        match html_body.state.as_ref() {
            Ok(state) => {
                vec![super::Message::OpenInEditor {
                    content: state.content.clone(),
                    ty: EditorContentType::Markdown,
                    on_exit: |result| match result {
                        Ok(_) => vec![],
                        Err(err) => {
                            vec![
                                Message::SetStatusbarMessage {
                                    msg: err.to_string(),
                                    ty: StatusMsgType::Error,
                                }
                                .into(),
                            ]
                        }
                    },
                }]
            }
            Err(err) => {
                self.statusbar
                    .set_message(err.to_string(), StatusMsgType::Error);
                vec![]
            }
        }
    }

    fn read_html_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        let Some(init_html_body) = self.html_body.as_ref() else {
            return vec![
                MessageRequest::GetMail {
                    username: self.ctx.username.clone(),
                    account_id: self.ctx.account_id.clone(),
                    mail_id: self.ctx.mail_id.clone(),
                    properties: vec![MailProperty::HtmlBody],
                    callback: Box::new(|cached_mail| match cached_mail {
                        Ok(_cached_mail) => {
                            vec![Message::UserAction(UserAction::ReadMarkdownBodyInEditor).into()]
                        }
                        Err(err) => vec![
                            Message::SetStatusbarMessage {
                                msg: format!("Couldn't fetch html body: {}", err),
                                ty: StatusMsgType::Error,
                            }
                            .into(),
                        ],
                    }),
                }
                .into(),
            ];
        };

        let Ok(html_body) = init_html_body else {
            return vec![];
        };

        vec![super::Message::OpenInEditor {
            content: html_body.html.clone(),
            ty: EditorContentType::Html,
            on_exit: |result| match result {
                Ok(_) => vec![],
                Err(err) => {
                    vec![
                        Message::SetStatusbarMessage {
                            msg: err.to_string(),
                            ty: StatusMsgType::Error,
                        }
                        .into(),
                    ]
                }
            },
        }]
    }

    fn download_attachment_and_paste_destination_path(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(None, SelectedTab::Attachments) {
            return vec![];
        }

        let Some(selected_attachment) = self
            .attachments
            .as_ref()
            .and_then(|tab| tab.get_selected_entry())
        else {
            return vec![];
        };

        let blob_id: BlobId = selected_attachment.blob_id.clone().into();
        let username = self.ctx.username.clone();
        let account_id = self.ctx.account_id.clone();

        self.statusbar.set_message(
            "[1/2] Downloading attachment".to_string(),
            StatusMsgType::Loading,
        );

        vec![crate::ui::Message::GetBlob {
            username,
            account_id,
            blob_id,
            callback: |result| {
                vec![Message::SaveAttachment(SaveAttachmentStep::GetDestinationPath(result)).into()]
            },
        }]
    }

    fn quit(&self) -> Vec<super::Message> {
        vec![super::Message::Quit]
    }

    fn back(&self) -> Vec<super::Message> {
        vec![super::Message::Back]
    }
}

enum SelectedBodyType {
    Text,
    Html,
}

struct Ctx {
    username: Username,
    account_id: AccountId,
    mail_id: MailId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum Mode {
    Reader,
    Composer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
enum SelectedTab {
    Body,
    Attachments,
}
