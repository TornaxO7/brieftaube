mod attachments_tab;
mod body;
mod message;
mod message_request;
mod types;
mod user_action;
mod view;

use crate::{
    config::Username,
    types::{AccountId, BlobId, CachedMail, MailAddresses, MailKeyword},
    ui::{
        EditorContentType, Layer, Loadable,
        pager::{attachments_tab::AttachmentsTab, message::SaveAttachmentStep},
        statusbar::{StatusMsgType, StatusbarState},
        utils::keybindmanager::{HandleEvent, KeybindManager},
    },
};
use body::*;
use crossterm::event::Event;
use mail_parser::MessageParser;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::OnceLock,
};
use throbber_widgets_tui::ThrobberState;
use tracing::{debug, error};
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

    ctx: UserCtx,
    content: Loadable<Content>,
}

impl State {
    pub fn new(username: Username, account_id: AccountId, mode: Mode) -> Self {
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

            ctx: UserCtx {
                username,
                account_id,
            },
            content: Loadable::Loading,
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
            Message::SetMail(result) => self.handle_set_mail(result),
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

            UserAction::EditBody => self.edit_body(),

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

    fn handle_set_mail(&mut self, result: color_eyre::Result<CachedMail>) -> Vec<super::Message> {
        match result {
            Ok(mail) => {
                let headers = MailHeaders::from(&mail);
                let text_body = TextBody::from(&mail);
                let html_body = HtmlBody::from(&mail);
                let attachments_tab = AttachmentsTab::from(&mail);

                self.content = Loadable::Loaded(Content {
                    mail,

                    headers,
                    text_body,
                    html_body,
                    attachments_tab,
                });
            }
            Err(err) => {
                error!("{:?}", err);

                self.content = Loadable::Error(
                    "Couldn't fetch mail. Please see the logs for more information".to_string(),
                );
            }
        };

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
        vec![]
    }

    fn open_html_body(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        self.selected_body_type = SelectedBodyType::Html;
        vec![]
    }

    fn navigate_down(&mut self) -> Vec<super::Message> {
        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    content.text_body.navigate_down(1);
                }
                SelectedBodyType::Html => {
                    content.html_body.navigate_down(1);
                }
            },
            SelectedTab::Attachments => {
                content.attachments_tab.navigate_down();
            }
        };
        vec![]
    }

    fn navigate_up(&mut self) -> Vec<super::Message> {
        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    content.text_body.navigate_up(1);
                }
                SelectedBodyType::Html => {
                    content.html_body.navigate_up(1);
                }
            },
            SelectedTab::Attachments => {
                content.attachments_tab.navigate_up();
            }
        };
        vec![]
    }

    fn navigate_to_top(&mut self) -> Vec<super::Message> {
        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    content.text_body.navigate_to_top();
                }
                SelectedBodyType::Html => {
                    content.html_body.navigate_to_top();
                }
            },
            SelectedTab::Attachments => {
                content.attachments_tab.navigate_to_top();
            }
        };

        vec![]
    }

    fn navigate_half_page_down(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Body) {
            return vec![];
        }

        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_body_type {
            SelectedBodyType::Text => {
                content.text_body.navigate_half_page_down();
            }
            SelectedBodyType::Html => {
                content.html_body.navigate_half_page_down();
            }
        }

        vec![]
    }

    fn navigate_half_page_up(&mut self) -> Vec<super::Message> {
        if !matches!(self.selected_tab, SelectedTab::Body) {
            return vec![];
        }

        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_body_type {
            SelectedBodyType::Text => {
                content.text_body.navigate_half_page_up();
            }
            SelectedBodyType::Html => {
                content.html_body.navigate_half_page_up();
            }
        }

        vec![]
    }

    fn navigate_to_bottom(&mut self) -> Vec<super::Message> {
        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        match self.selected_tab {
            SelectedTab::Body => match self.selected_body_type {
                SelectedBodyType::Text => {
                    content.text_body.navigate_to_bottom();
                }
                SelectedBodyType::Html => {
                    content.html_body.navigate_to_bottom();
                }
            },
            SelectedTab::Attachments => {
                content.attachments_tab.navigate_to_bottom();
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

    fn edit_body(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Composer), SelectedTab::Body) {
            return vec![];
        }

        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        let current_mail = content.mail.clone();
        let headers_string = content.mail.headers_as_string();

        let (content, ty): (String, super::EditorContentType) = match self.selected_body_type {
            SelectedBodyType::Text => {
                let ty = super::EditorContentType::MailText;
                let content = format!(
                    "{}\n{}",
                    headers_string,
                    content.text_body.state.content.clone(),
                );
                (content, ty)
            }
            SelectedBodyType::Html => {
                let ty = super::EditorContentType::Markdown;
                let content = format!(
                    "{}\n{}",
                    headers_string,
                    content.html_body.ctx.as_ref().unwrap().markdown.clone()
                );
                (content, ty)
            }
        };

        let ctx = self.ctx.clone();
        let body_type = self.selected_body_type.clone();
        vec![super::Message::OpenInEditor {
            content,
            ty,
            on_exit: Box::new(move |result| match result {
                Ok(new_content) => {
                    let parsed_mail = MessageParser::new().parse(&new_content).unwrap();

                    let new_mail = CachedMail {
                        keywords: match parsed_mail.keywords().as_text_list() {
                            Some(keywords) => {
                                let keywords: HashSet<MailKeyword> =
                                    keywords.into_iter().map(MailKeyword::from).collect();
                                OnceLock::from(keywords)
                            }
                            None => OnceLock::from(HashSet::from([
                                MailKeyword::Draft,
                                MailKeyword::Seen,
                            ])),
                        },
                        subject: match parsed_mail.subject() {
                            Some(subject) => OnceLock::from(subject.to_string()),
                            None => OnceLock::from(String::new()),
                        },
                        from: parsed_mail.from().map_or_default(|from| match from {
                            mail_parser::Address::List(addrs) => {
                                let addresses = MailAddresses::from(addrs);
                                OnceLock::from(addresses)
                            }
                            mail_parser::Address::Group(_groups) => todo!(),
                        }),
                        to: parsed_mail.to().map_or_default(|to| match to {
                            mail_parser::Address::List(addrs) => {
                                let addresses = MailAddresses::from(addrs);
                                OnceLock::from(addresses)
                            }
                            mail_parser::Address::Group(_groups) => todo!(),
                        }),
                        cc: parsed_mail.cc().map_or_default(|cc| match cc {
                            mail_parser::Address::List(addrs) => {
                                let addresses = MailAddresses::from(addrs);
                                OnceLock::from(addresses)
                            }
                            mail_parser::Address::Group(_groups) => todo!(),
                        }),
                        bcc: parsed_mail.bcc().map_or_default(|bcc| match bcc {
                            mail_parser::Address::List(addrs) => {
                                let addresses = MailAddresses::from(addrs);
                                OnceLock::from(addresses)
                            }
                            mail_parser::Address::Group(_groups) => todo!(),
                        }),
                        ..current_mail.clone()
                    };

                    // overwrite body
                    let has_empty_body = parsed_mail.text_body_count() == 0;
                    let new_mail = if has_empty_body {
                        new_mail
                    } else {
                        let mail_parser::PartType::Text(text_body_content) =
                            &parsed_mail.text_part(0).unwrap().body
                        else {
                            unreachable!();
                        };

                        let text_body_content = text_body_content.to_string();

                        let part_id = std::iter::repeat_with(fastrand::alphanumeric)
                            .take(10)
                            .collect::<String>();

                        match body_type {
                            SelectedBodyType::Text => {
                                let body_parts =
                                    HashMap::from([(part_id.clone(), text_body_content)]);

                                CachedMail {
                                    text_part_ids: OnceLock::from(vec![part_id]),
                                    body_parts: OnceLock::from(body_parts),
                                    ..new_mail
                                }
                            }
                            SelectedBodyType::Html => {
                                let html = {
                                    let mut html_output = String::new();

                                    pulldown_cmark::html::push_html(
                                        &mut html_output,
                                        pulldown_cmark::Parser::new_ext(
                                            &text_body_content,
                                            pulldown_cmark::Options::all(),
                                        ),
                                    );

                                    html_output
                                };

                                let body_parts = HashMap::from([(part_id.clone(), html)]);

                                CachedMail {
                                    html_part_ids: OnceLock::from(vec![part_id]),
                                    body_parts: OnceLock::from(body_parts),
                                    ..new_mail
                                }
                            }
                        }
                    };

                    vec![crate::ui::Message::EditDraftMail {
                        username: ctx.username.clone(),
                        account_id: ctx.account_id.clone(),
                        new_draft,
                    }]
                }
                Err(err) => {
                    let msg = format!("Can't save new body: {}", err);
                    error!("{}", msg);

                    vec![
                        Message::SetStatusbarMessage {
                            msg,
                            ty: StatusMsgType::Error,
                        }
                        .into(),
                    ]
                }
            }),
        }]
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

        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        let headers = content.mail.headers_as_string();
        let content = format!("{}\n{}", headers, &content.text_body.state.content);

        vec![super::Message::OpenInEditor {
            content,
            ty: EditorContentType::MailText,
            on_exit: Box::new(|_| vec![]),
        }]
    }

    fn read_markdown_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        let Some(content) = self.content.loaded_mut() else {
            return vec![];
        };

        let headers = content.mail.headers_as_string();
        let content = format!(
            "{}\n{}",
            headers,
            &content.html_body.ctx.as_ref().unwrap().markdown
        );

        vec![super::Message::OpenInEditor {
            content,
            ty: EditorContentType::Markdown,
            on_exit: Box::new(|_| vec![]),
        }]
    }

    fn read_html_body_in_editor(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(Some(Mode::Reader), SelectedTab::Body) {
            return vec![];
        }

        let Some(content) = self.content.loaded() else {
            return vec![];
        };

        let headers = content.mail.headers_as_string();
        let content = format!("{}\n{}", headers, &content.html_body.html);

        vec![super::Message::OpenInEditor {
            content,
            ty: EditorContentType::Html,
            on_exit: Box::new(|result| match result {
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
            }),
        }]
    }

    fn download_attachment_and_paste_destination_path(&mut self) -> Vec<super::Message> {
        if !self.is_in_state(None, SelectedTab::Attachments) {
            return vec![];
        }

        let Some(content) = self.content.loaded() else {
            return vec![];
        };

        let Some(selected_attachment) = content.attachments_tab.get_selected_entry() else {
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

#[derive(Debug, Clone, Copy)]
enum SelectedBodyType {
    Text,
    Html,
}

#[derive(Debug, Clone)]
struct UserCtx {
    username: Username,
    account_id: AccountId,
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

struct Content {
    mail: CachedMail,

    headers: MailHeaders,
    text_body: TextBody,
    html_body: HtmlBody,
    attachments_tab: AttachmentsTab,
}
