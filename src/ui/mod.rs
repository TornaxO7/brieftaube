mod repository_manager;
mod task_manager;
mod types;
mod utils;

pub mod mailfs;
pub mod pager;
pub mod palette;
pub mod prompt;
pub mod statusbar;

use crate::{
    CONFIG, THEME,
    config::Username,
    types::{AccountId, CachedMail, MailId, MailProperty},
    ui::{palette::PaletteEntry, repository_manager::RepositoryManager},
};
use crossterm::{
    ExecutableCommand,
    event::Event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::{FutureExt, StreamExt};
use material_theme_loader::Scheme;
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;
use tracing::error;

pub use types::*;

pub enum Message {
    Event(Event),
    Navigation(Navigation),

    Mailfs(mailfs::Message),
    MailfsRequest(mailfs::MessageRequest),

    Pager(pager::Message),
    PagerRequest(pager::MessageRequest),
}

/// Stores the app state
pub struct Ui {
    is_running: bool,
    layers: Vec<ActiveLayer>,
    scheme: Scheme,
    pending_editor: Option<PendingEditor>,
    manager: RepositoryManager,
}

impl Ui {
    pub fn new() -> Self {
        let mut manager = RepositoryManager::new();

        let (mailfs, initial_user) = mailfs::State::new();
        manager.load_user(initial_user);

        let scheme = {
            let theme = THEME.get().unwrap();
            theme.schemes.dark.clone()
        };

        Self {
            is_running: true,
            layers: vec![ActiveLayer::Mailfs(mailfs)],
            scheme,
            pending_editor: None,

            manager,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::eyre::Result<()> {
        let mut msgs = Vec::with_capacity(8);
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut event_stream = crossterm::event::EventStream::new();

        terminal.draw(|frame| self.draw(frame, &mut msgs))?;

        while self.is_running {
            tokio::select! {
                maybe_event = event_stream.next().fuse() => match maybe_event {
                    Some(Ok(event)) => msgs.push(Message::Event(event)),
                    Some(Err(e)) => error!("{}", e),
                    None => (),
                },
                next_message = self.manager.finish_next_task(), if self.manager.has_tasks_running() => {
                    msgs.extend(next_message);
                },
                _timeout = interval.tick(), if self.manager.has_tasks_running() => {}
            };

            while let Some(next_message) = msgs.pop() {
                msgs.extend(self.handle_message(next_message));
            }

            if let Some(pending_editor) = self.pending_editor.take() {
                // We need to drop the event stream because `crossterm` starts a backend-thread which listens on `stdin`
                // and conflicts with the upcoming editor.
                drop(event_stream);
                msgs.extend(pending_editor.run());
                event_stream = crossterm::event::EventStream::new();

                terminal
                    .draw(|frame| frame.render_widget(ratatui::widgets::Clear, frame.area()))?;

                while let Some(next_message) = msgs.pop() {
                    msgs.extend(self.handle_message(next_message));
                }
            }

            terminal.draw(|frame| self.draw(frame, &mut msgs))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, msgs: &mut Vec<Message>) {
        let area = frame.area();

        if matches!(self.layers.last().unwrap(), ActiveLayer::Overlay(_)) {
            msgs.extend(match self.layers.iter_mut().rev().skip(1).next().unwrap() {
                ActiveLayer::Mailfs(state) => mailfs::view(&self.scheme, state, frame, area),
                ActiveLayer::Pager(state) => pager::view(&self.scheme, state, frame, area),
                ActiveLayer::Overlay(_) => unreachable!(),
            });
        }

        msgs.extend(match self.layers.last_mut().unwrap() {
            ActiveLayer::Mailfs(state) => mailfs::view(&self.scheme, state, frame, area),
            ActiveLayer::Pager(state) => pager::view(&self.scheme, state, frame, area),
            ActiveLayer::Overlay(overlay) => match overlay {
                OverlayLayer::Palette(state) => palette::view(&self.scheme, state, frame, area),
                OverlayLayer::Prompt(state) => prompt::view(&self.scheme, state, frame, area),
            },
        });
    }

    fn handle_message(&mut self, msg: Message) -> Vec<Message> {
        match msg {
            Message::Event(event) => match self.layers.last_mut().unwrap() {
                ActiveLayer::Mailfs(state) => state.update(mailfs::Message::Event(event)),
                ActiveLayer::Pager(state) => state.update(pager::Message::Event(event)),
                ActiveLayer::Overlay(overlay) => match overlay {
                    OverlayLayer::Palette(state) => state.update(palette::Message::Event(event)),
                    OverlayLayer::Prompt(state) => state.update(prompt::Message::Event(event)),
                },
            },

            Message::Navigation(navigation) => match navigation {
                Navigation::Back => self.navigate_back(),
                Navigation::Quit => self.navigate_quit(),
                Navigation::OpenPrompt { description, map } => {
                    self.navigate_open_prompt(description, map)
                }
                Navigation::OpenPalette { entries, map } => {
                    self.navigate_open_palette(entries, map)
                }
                Navigation::OpenReader {
                    username,
                    account_id,
                    mail_id,
                } => self.navigate_open_reader(username, account_id, mail_id),
                Navigation::OpenComposer {
                    username,
                    account_id,
                    mail_id,
                } => self.navigate_open_composer(username, account_id, mail_id),
                Navigation::OpenEditor {
                    content,
                    ty,
                    callback,
                } => self.navigate_open_editor(content, ty, callback),
            },

            Message::Mailfs(msg) => self.handle_mailfs(msg),
            Message::MailfsRequest(msg) => mailfs::handle_request(msg, &mut self.manager),

            Message::Pager(msg) => self.handle_pager(msg),
            Message::PagerRequest(msg) => pager::handle_request(msg, &mut self.manager),
        }
    }
}

// Navigation
impl Ui {
    fn navigate_back(&mut self) -> Vec<Message> {
        self.layers.pop();
        vec![]
    }

    fn navigate_quit(&mut self) -> Vec<Message> {
        self.is_running = false;
        vec![]
    }

    fn navigate_open_prompt(
        &mut self,
        description: String,
        map: Box<dyn Fn(String) -> Message + Send>,
    ) -> Vec<Message> {
        let layer =
            ActiveLayer::Overlay(OverlayLayer::Prompt(prompt::State::new(description, map)));

        self.layers.push(layer);
        vec![]
    }

    fn navigate_open_palette(
        &mut self,
        entries: Vec<PaletteEntry>,
        map: fn(String) -> Message,
    ) -> Vec<Message> {
        let overlay = OverlayLayer::Palette(palette::State::new(entries, map));
        self.layers.push(ActiveLayer::Overlay(overlay));
        vec![]
    }

    fn navigate_open_reader(
        &mut self,
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    ) -> Vec<Message> {
        self.layers.push(ActiveLayer::Pager(pager::State::new(
            username.clone(),
            account_id.clone(),
            pager::Mode::Reader,
        )));

        let properties = {
            let mut properties = pager::MailHeaders::MAIL_PROPERTIES.to_vec();
            properties.extend([
                MailProperty::HtmlBody,
                MailProperty::TextBody,
                MailProperty::Attachments,
            ]);
            properties
        };

        vec![
            pager::MessageRequest::GetMail {
                username,
                account_id,
                mail_id,
                properties,
                callback: Box::new(move |cached_mail: color_eyre::Result<CachedMail>| {
                    vec![pager::Message::SetMail(cached_mail).into()]
                }),
            }
            .into(),
        ]
    }

    fn navigate_open_composer(
        &mut self,
        username: Username,
        account_id: AccountId,
        mail_id: Option<MailId>,
    ) -> Vec<Message> {
        match mail_id {
            Some(_) => todo!("Open already existing mailid"),
            None => {
                let mail_id = MailId::new_intern();

                self.layers.push(ActiveLayer::Pager(pager::State::new(
                    username.clone(),
                    account_id.clone(),
                    pager::Mode::Composer,
                )));

                self.manager.with_handler(&username, async move |handler| {
                    vec![
                        pager::Message::SetMail(handler.create_new_mail(account_id, mail_id).await)
                            .into(),
                    ]
                });

                vec![]
            }
        }
    }

    fn navigate_open_editor(
        &mut self,
        content: String,
        ty: EditorContentType,
        callback: Box<dyn Fn(Result<String, OpenEditorError>) -> Vec<Message> + Send>,
    ) -> Vec<Message> {
        self.pending_editor = Some(PendingEditor {
            content,
            ty,
            callback,
        });

        vec![]
    }
}

// Layers
impl Ui {
    fn handle_mailfs(&mut self, msg: mailfs::Message) -> Vec<Message> {
        self.layers
            .iter_mut()
            .rev()
            .find_map(|layer| {
                if let ActiveLayer::Mailfs(state) = layer {
                    Some(state)
                } else {
                    None
                }
            })
            .map(|mailfs| mailfs.update(msg).into_iter().map(Message::from).collect())
            .unwrap_or_default()
    }

    fn handle_pager(&mut self, msg: pager::Message) -> Vec<Message> {
        self.layers
            .iter_mut()
            .rev()
            .find_map(|layer| {
                if let ActiveLayer::Pager(pager) = layer {
                    Some(pager)
                } else {
                    None
                }
            })
            .map(|pager| pager.update(msg))
            .unwrap_or_default()
    }
}

pub trait Layer<LayerMsg, ParentMsg = Message> {
    #[must_use]
    fn update(&mut self, msg: LayerMsg) -> Vec<ParentMsg>;
}

enum ActiveLayer {
    Mailfs(mailfs::State),
    Pager(pager::State),
    Overlay(OverlayLayer),
}

enum OverlayLayer {
    Palette(palette::State),
    Prompt(prompt::State),
}

#[derive(Debug, Clone, Copy)]
pub enum EditorContentType {
    MailText,
    Markdown,
    Html,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenEditorError {
    #[error("")]
    NoEditorFound,
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

struct PendingEditor {
    content: String,
    ty: EditorContentType,
    callback: Box<dyn Fn(Result<String, OpenEditorError>) -> Vec<Message> + Send>,
}

impl PendingEditor {
    fn run(self) -> Vec<Message> {
        (self.callback)(Self::open_editor(self.content, self.ty))
    }

    fn open_editor(content: String, ty: EditorContentType) -> Result<String, OpenEditorError> {
        let editor = CONFIG
            .get()
            .unwrap()
            .editor()
            .ok_or(OpenEditorError::NoEditorFound)?;

        let tmp_file_path = {
            let ending = match ty {
                EditorContentType::MailText => "eml",
                EditorContentType::Markdown => "md",
                EditorContentType::Html => "html",
            };

            let filename = format!(
                "tmp-{}.{}",
                std::iter::repeat_with(fastrand::alphanumeric)
                    .take(10)
                    .collect::<String>(),
                ending
            );

            crate::get_runtime_file_path(filename)?
        };

        std::fs::write(&tmp_file_path, content)?;
        disable_raw_mode()?;
        std::io::stdout().execute(LeaveAlternateScreen)?;

        std::process::Command::new(editor)
            .arg(&tmp_file_path)
            .status()?;

        std::io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        let editor_content = std::fs::read_to_string(&tmp_file_path)?;

        // oh no, we couldn't remove the temporary file... anyways
        let _ignore = std::fs::remove_file(tmp_file_path);

        Ok(editor_content)
    }
}

pub enum Navigation {
    Back,
    Quit,

    OpenPrompt {
        description: String,
        map: Box<dyn Fn(String) -> Message + Send>,
    },
    OpenPalette {
        entries: Vec<PaletteEntry>,
        map: fn(String) -> Message,
    },
    OpenReader {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    },
    OpenComposer {
        username: Username,
        account_id: AccountId,
        mail_id: Option<MailId>,
    },
    OpenEditor {
        content: String,
        ty: EditorContentType,

        callback: Box<dyn Fn(Result<String, OpenEditorError>) -> Vec<Message> + Send>,
    },
}

impl From<Navigation> for Message {
    fn from(navigation: Navigation) -> Self {
        Self::Navigation(navigation)
    }
}
