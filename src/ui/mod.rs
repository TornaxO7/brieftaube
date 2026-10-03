mod renderer;
mod task_manager;
mod types;
mod utils;

pub mod mailfs;
pub mod pager;
pub mod palette;
pub mod prompt;
pub mod statusbar;

use material_theme_loader::Scheme;
use tokio::sync::{RwLock, watch};
pub use types::*;

use crate::{
    THEME,
    config::{self, Username},
    datasource::{self, Cache, RemoteSession, jmap::JmapDescriptor},
    repository::RepositoryHandler,
    types::{AccountId, MailId},
    ui::palette::PaletteEntry,
};
use color_eyre::eyre;
use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use ratatui::{DefaultTerminal, Frame, layout::Rect};
use std::{collections::HashMap, time::Duration};
use task_manager::TaskManager;
use tracing::error;

pub enum Message {
    Mailfs(mailfs::Message),
    MailfsRequest(mailfs::MessageRequest),

    Pager(pager::Message),
    PagerRequest(pager::MessageRequest),

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
    OpenPager {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
        mode: pager::Mode,
    },

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
    scheme: Scheme,

    repos: HashMap<Username, watch::Receiver<RepositoryState>>,
}

impl Ui {
    pub fn new(init_rect: Rect) -> Self {
        let task_manager = TaskManager::new();

        let (mailfs, initial_user) = mailfs::State::new();

        task_manager.spawn(init_user(initial_user));

        let scheme = {
            let theme = THEME.get().unwrap();
            theme.schemes.dark.clone()
        };

        Self {
            repos: HashMap::new(),

            is_running: true,
            layers: vec![ActiveLayer::Mailfs(mailfs)],
            needs_full_redraw: false,
            task_manager,
            scheme,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> eyre::Result<()> {
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
                next_message = self.task_manager.finish_next_task(), if self.task_manager.has_tasks_running() => {
                    msgs.extend(next_message);
                },
                _timeout = interval.tick(), if self.task_manager.has_tasks_running() => {}
            };

            while let Some(next_message) = msgs.pop() {
                msgs.extend(self.handle_message(next_message));
            }

            terminal.draw(|frame| self.draw(frame, &mut msgs))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, msgs: &mut Vec<Message>) {
        let area = frame.area();

        if matches!(self.layers.last(), Some(ActiveLayer::Overlay(_))) {
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

            Message::OpenPrompt { description, map } => {
                self.layers.push(ActiveLayer::Overlay(OverlayLayer::Prompt(
                    prompt::State::new(description, map),
                )));
                vec![]
            }
            Message::OpenPalette { entries, map } => {
                let overlay = OverlayLayer::Palette(palette::State::new(entries, map));
                self.layers.push(ActiveLayer::Overlay(overlay));
                vec![]
            }
            Message::OpenPager {
                username,
                account_id,
                mail_id,
                mode,
            } => {
                self.layers.push(ActiveLayer::Pager(pager::State::new(
                    username.clone(),
                    account_id.clone(),
                    mail_id.clone(),
                    mode,
                )));

                vec![
                    Message::PagerRequest(pager::MessageRequest::GetHeaders {
                        username: username.clone(),
                        account_id: account_id.clone(),
                        mail_id: mail_id.clone(),
                    }),
                    Message::PagerRequest(pager::MessageRequest::GetHtmlBody {
                        username: username.clone(),
                        account_id: account_id.clone(),
                        mail_id: mail_id.clone(),
                    }),
                    Message::PagerRequest(pager::MessageRequest::GetAttachments {
                        username: username.clone(),
                        account_id: account_id.clone(),
                        mail_id: mail_id.clone(),
                    }),
                ]
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
            Message::Mailfs(message) => {
                let mailfs = self
                    .layers
                    .iter_mut()
                    .rev()
                    .find_map(|layer| {
                        if let ActiveLayer::Mailfs(state) = layer {
                            Some(state)
                        } else {
                            None
                        }
                    })
                    .expect("Mailfs is in layer");

                mailfs.update(message);
                vec![]
            }
            Message::MailfsRequest(message_request) => {
                match message_request {
                    mailfs::MessageRequest::GetAccountsOf(user_config) => {
                        let (tx, rx) = watch::channel(RepositoryState::Loading);
                        self.repos.insert(user_config.username.clone(), rx);
                        self.task_manager
                            .spawn(mailfs_repository_create(user_config, tx));
                    }
                    mailfs::MessageRequest::InitMailbox {
                        username,
                        account_id,
                        mailbox_id,
                        max_init_mails: amount_initial_mails,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                mailfs::Message::InitMailbox {
                                    username,
                                    account_id: account_id.clone(),
                                    mailbox_id: mailbox_id.clone(),

                                    data: handler
                                        .get_init_mailbox(
                                            account_id,
                                            mailbox_id,
                                            amount_initial_mails,
                                        )
                                        .await,
                                }
                                .into(),
                            ]
                        });
                    }
                    mailfs::MessageRequest::QueryMails {
                        username,
                        account_id,
                        mailbox,
                        window,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                mailfs::Message::SetMails {
                                    username,
                                    account_id: account_id.clone(),
                                    mailbox: mailbox.clone(),
                                    window: window.clone(),
                                    result: handler.query_mails(account_id, mailbox, window).await,
                                }
                                .into(),
                            ]
                        });
                    }
                    mailfs::MessageRequest::GetThreadMails {
                        username,
                        account_id,
                        thread_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                mailfs::Message::SetThreadMails {
                                    username,
                                    account_id: account_id.clone(),
                                    thread_id: thread_id.clone(),
                                    thread_mails: handler
                                        .get_thread_mails(account_id, thread_id)
                                        .await,
                                }
                                .into(),
                            ]
                        });
                    }
                    mailfs::MessageRequest::GetMailPreview {
                        username,
                        account_id,
                        mail_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                mailfs::Message::SetMailPreview {
                                    username,
                                    account_id: account_id.clone(),
                                    mail_id: mail_id.clone(),
                                    preview: handler.get_mail_preview(account_id, mail_id).await,
                                }
                                .into(),
                            ]
                        });
                    }
                };
                vec![]
            }

            Message::Pager(message) => {
                let pager = self
                    .layers
                    .iter_mut()
                    .rev()
                    .find_map(|layer| {
                        if let ActiveLayer::Pager(pager) = layer {
                            Some(pager)
                        } else {
                            None
                        }
                    })
                    .expect("Pager is in `layers`");

                pager.update(message);
                vec![]
            }
            Message::PagerRequest(message_request) => {
                match message_request {
                    pager::MessageRequest::GetHeaders {
                        username,
                        account_id,
                        mail_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            let headers = handler.get_mail_preview(account_id, mail_id).await.map(
                                |preview| pager::MailHeaders {
                                    from: preview.from.map(|from| from.to_string()),
                                    to: preview.to.map(|to| to.to_string()),
                                    cc: preview.cc.map(|cc| cc.to_string()),
                                    subject: preview.subject,
                                    received_at: preview
                                        .received_at
                                        .format("%b %e, %Y")
                                        .to_string(),
                                },
                            );

                            vec![Message::Pager(pager::Message::SetHeaders(headers)).into()]
                        });
                    }
                    pager::MessageRequest::GetTextBody {
                        username,
                        account_id,
                        mail_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                pager::Message::SetTextBody(
                                    handler.get_mail_text_body(account_id, mail_id).await,
                                )
                                .into(),
                            ]
                        });
                    }
                    pager::MessageRequest::GetHtmlBody {
                        username,
                        account_id,
                        mail_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                pager::Message::SetHtmlBody(
                                    handler.get_mail_html_body(account_id, mail_id).await,
                                )
                                .into(),
                            ]
                        });
                    }
                    pager::MessageRequest::GetAttachments {
                        username,
                        account_id,
                        mail_id,
                    } => {
                        let state = self.repos.get(&username).unwrap().clone();

                        self.task_manager.spawn(async move {
                            let handler = match get_handler(state).await {
                                Ok(handler) => handler,
                                Err(()) => return vec![],
                            };

                            vec![
                                pager::Message::SetAttachments(
                                    handler
                                        .get_mail_preview(account_id, mail_id)
                                        .await
                                        .map(|preview| preview.attachments.unwrap_or(vec![])),
                                )
                                .into(),
                            ]
                        });
                    }
                };
                vec![]
            }

            Message::Palette(message) => {
                let palette = self
                    .layers
                    .iter_mut()
                    .rev()
                    .find_map(|layer| {
                        if let ActiveLayer::Overlay(OverlayLayer::Palette(palette)) = layer {
                            Some(palette)
                        } else {
                            None
                        }
                    })
                    .expect("Palette is in active layers");

                palette.update(message);
                vec![]
            }
            Message::Prompt(message) => {
                let prompt = self
                    .layers
                    .iter_mut()
                    .rev()
                    .find_map(|layer| {
                        if let ActiveLayer::Overlay(OverlayLayer::Prompt(prompt)) = layer {
                            Some(prompt)
                        } else {
                            None
                        }
                    })
                    .expect("prompt is in active layers");

                prompt.update(message);
                vec![]
            }
        }
    }
}

pub trait Layer<LayerMsg, ParentLayerMsg = Message> {
    fn update(&mut self, msg: LayerMsg) -> Vec<ParentLayerMsg>;
}

async fn mailfs_repository_create(
    user_config: config::UserConfig,
    tx: watch::Sender<RepositoryState>,
) -> Vec<Message> {
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

                    tx.send_replace(RepositoryState::Error(err.to_string()));

                    return vec![
                        mailfs::Message::SetUserAccounts {
                            username: user_config.username.clone(),
                            accounts: Err(err.into()),
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

    tx.send_replace(RepositoryState::Loaded(RepositoryHandler::new(
        caches, remote,
    )));

    vec![
        mailfs::Message::SetUserAccounts {
            username: user_config.username.clone(),
            accounts: Ok(accounts),
        }
        .into(),
    ]
}

async fn get_handler(
    mut waiter: watch::Receiver<RepositoryState>,
) -> Result<RepositoryHandler, ()> {
    let state = waiter
        .wait_for(|current_state| !matches!(current_state, RepositoryState::Loading))
        .await
        .expect("Repository creation should finish.")
        .clone();

    match state {
        RepositoryState::Loading => unreachable!(),
        RepositoryState::Loaded(handler) => Ok(handler.clone()),
        RepositoryState::Error(err) => {
            error!("Can't get handler: {err}");
            Err(())
        }
    }
}

#[derive(Clone)]
enum RepositoryState {
    Loading,
    Loaded(RepositoryHandler),
    Error(String),
}

async fn init_user(user: config::UserConfig) -> Vec<Message> {
    vec![Message::MailfsRequest(
        mailfs::MessageRequest::GetAccountsOf(user),
    )]
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
