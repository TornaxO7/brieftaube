mod renderer;
mod task_manager;
mod types;
mod utils;

// pub mod composer;
// pub mod log_viewer;
pub mod mailfs;
pub mod palette;
pub mod prompt;
// pub mod reader;
pub mod statusbar;

use tokio::sync::RwLock;
pub use types::*;

use crate::{
    CONFIG,
    config::{self, Username},
    datasource::{self, Cache, RemoteAccount, RemoteSession, jmap::JmapDescriptor},
    repository::RepositoryHandler,
    ui::palette::PaletteEntry,
};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame};
use std::{collections::HashMap, time::Duration};
use task_manager::TaskManager;
use tracing::error;

#[derive(Debug, Clone, Copy)]
enum ActiveLayer {
    Mailfs,
    Palette,
    Prompt,
}

pub enum Message {
    Mailfs(mailfs::Message),
    MailfsRequest(mailfs::MessageRequest),

    Palette(palette::Message),
    Prompt(prompt::Message),

    Event(Event),
    OpenPrompt {
        description: String,
        map: fn(String) -> Message,
    },
    OpenPalette {
        entries: Vec<PaletteEntry>,
        map: fn(String) -> Message,
    },

    AddRepositoryHandler(Username, RepositoryHandler),
    Back,
    Redraw,
    Quit,
}

/// Stores the app state
// IDEA: Use mpsc::Receiver or so and a sender to each ui module => Just send them instead of allocating `vec![]` all the time to send messages
pub struct Ui {
    is_running: bool,
    layers: Vec<ActiveLayer>,
    needs_full_redraw: bool,
    task_manager: TaskManager,

    repos: HashMap<Username, RepositoryHandler>,

    mailfs: mailfs::State,
    palette: palette::State,
    prompt: prompt::State,
}

impl Ui {
    pub fn new() -> Self {
        let task_manager = TaskManager::new();

        let mailfs = mailfs::State::new();
        let palette = palette::State::new();
        let prompt = prompt::State::new();

        Self {
            mailfs,
            palette,
            prompt,

            repos: HashMap::new(),

            is_running: true,
            layers: vec![ActiveLayer::Mailfs],
            needs_full_redraw: false,
            task_manager,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
        let mut reader = crossterm::event::EventStream::new();
        terminal.draw(|frame| self.draw(frame))?;

        let mut msgs = Vec::with_capacity(8);
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        while self.is_running {
            tokio::select! {
                maybe_event = reader.next().fuse() => match maybe_event {
                    Some(Ok(event)) => msgs.push(Message::Event(event)),
                    Some(Err(e)) => error!("{}", e),
                    None => (),
                },
                next_message = self.task_manager.finish_next_task(), if self.task_manager.has_tasks_running() => {
                    msgs.extend(next_message);
                },
                _timeout = interval.tick(), if self.task_manager.has_tasks_running() => {}
            };

            while let Some(next_message) = msgs.pop() {
                msgs.extend(self.handle_message(next_message));
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let is_overlay = match self.layers.last().unwrap() {
            ActiveLayer::Mailfs => false,
            ActiveLayer::Palette | ActiveLayer::Prompt => true,
        };

        if is_overlay {
            match self.layers.iter().rev().skip(1).next().unwrap() {
                ActiveLayer::Mailfs => mailfs::view(&mut self.mailfs, frame, area),
                ActiveLayer::Palette => palette::view(&mut self.palette, frame, area),
                ActiveLayer::Prompt => prompt::view(&mut self.prompt, frame, area),
            }
        }

        match self.layers.last_mut().unwrap() {
            ActiveLayer::Mailfs => mailfs::view(&mut self.mailfs, frame, area),
            ActiveLayer::Palette => palette::view(&mut self.palette, frame, area),
            ActiveLayer::Prompt => prompt::view(&mut self.prompt, frame, area),
        }
    }

    fn handle_message(&mut self, msg: Message) -> Vec<Message> {
        match msg {
            Message::Event(event) => match self.layers.last_mut().unwrap() {
                ActiveLayer::Mailfs => self.mailfs.update(mailfs::Message::Event(event)),
                ActiveLayer::Palette => self.palette.update(palette::Message::Event(event)),
                ActiveLayer::Prompt => self.prompt.update(prompt::Message::Event(event)),
            },

            Message::AddRepositoryHandler(username, handler) => {
                self.repos.insert(username.clone(), handler);
                vec![]
            }

            Message::OpenPrompt { description, map } => {
                self.prompt
                    .update(prompt::Message::Reset { description, map });
                self.layers.push(ActiveLayer::Prompt);
                vec![]
            }
            Message::OpenPalette { entries, map } => {
                self.palette
                    .update(palette::Message::Restart { entries, map });
                self.layers.push(ActiveLayer::Palette);
                vec![]
            }

            Message::Back => {
                self.layers.pop();
                vec![]
            }

            Message::Redraw => {
                self.needs_full_redraw = true;
                vec![]
            }

            Message::Quit => {
                self.is_running = false;
                vec![]
            }
            Message::Mailfs(message) => self.mailfs.update(message),
            Message::MailfsRequest(message_request) => {
                match message_request {
                    mailfs::MessageRequest::GetAccountsOf(user_config) => {
                        self.task_manager
                            .spawn(mailfs_repository_create(user_config));
                    }
                    mailfs::MessageRequest::RepositoryCommand { user, command } => todo!(),
                    mailfs::MessageRequest::GetChildMailboxes { account_id, parent } => todo!(),
                    mailfs::MessageRequest::QueryMails {
                        account_id,
                        mailbox,
                        window,
                    } => todo!(),
                    mailfs::MessageRequest::GetThreadMails { account_id, thread } => todo!(),
                };
                vec![]
            }

            Message::Palette(message) => self.palette.update(message),
            Message::Prompt(message) => self.prompt.update(message),
        }
    }
}

pub trait Layer<LayerMsg, ParentLayerMsg = Message> {
    fn update(&mut self, msg: LayerMsg) -> Vec<ParentLayerMsg>;
}

async fn mailfs_repository_create(user_config: config::UserConfig) -> Vec<Message> {
    let remote = match user_config.backend {
        config::Backend::Jmap => {
            let desc = JmapDescriptor {
                credentials: jmap_client::client::Credentials::basic(
                    user_config.username.as_str(),
                    &user_config.password,
                ),
                server_url: user_config.server_url.clone(),
            };

            match datasource::jmap::JmapSession::connect(desc).await {
                Ok(remote) => Box::new(remote) as Box<dyn RemoteSession>,
                Err(err) => {
                    error!("Couldn't connect to jmap account: {}", err);

                    return vec![
                        mailfs::Message::SetUserAccounts {
                            username: user_config.username.clone(),
                            accounts: Loadable::Error(err.to_string()),
                        }
                        .into(),
                    ];
                }
            }
        }
    };

    let accounts = remote.get_accounts();

    let caches = accounts
        .clone()
        .into_iter()
        .map(|account| {
            let cache: Box<dyn Cache> = match user_config.cache {
                config::Cache::Internal => {
                    Box::new(datasource::hashmap::HashMapDataSource::new()) as Box<dyn Cache>
                }
            };

            (account.id, RwLock::new(cache))
        })
        .collect();

    let handler = RepositoryHandler::new(caches, remote);

    vec![
        Message::AddRepositoryHandler(user_config.username.clone(), handler),
        mailfs::Message::SetUserAccounts {
            username: user_config.username.clone(),
            accounts: Loadable::Loaded(accounts),
        }
        .into(),
    ]
}
