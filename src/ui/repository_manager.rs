use crate::{
    CONFIG,
    config::{self, UserConfig, Username},
    datasource::{self, Cache, RemoteSession, jmap::JmapDescriptor},
    repository::RepositoryHandler,
    ui::{Message, mailfs, task_manager::TaskManager},
};
use std::collections::HashMap;
use tokio::sync::{RwLock, watch};
use tracing::error;

pub struct RepositoryManager {
    task_manager: TaskManager,
    repos: HashMap<Username, watch::Receiver<RepositoryState>>,
}

impl RepositoryManager {
    pub fn new() -> Self {
        let task_manager = TaskManager::new();
        let repos = HashMap::new();

        Self {
            task_manager,
            repos,
        }
    }

    pub async fn finish_next_task(&self) -> Vec<Message> {
        self.task_manager.finish_next_task().await
    }

    pub fn has_tasks_running(&self) -> bool {
        self.task_manager.has_tasks_running()
    }

    pub fn load_user(&mut self, username: Username) {
        let user_config = CONFIG
            .get()
            .unwrap()
            .users
            .iter()
            .find(|user_config| user_config.username == username)
            .cloned()
            .unwrap();

        let (tx, rx) = watch::channel(RepositoryState::Loading);
        self.repos.insert(user_config.username.clone(), rx);
        self.task_manager
            .spawn(mailfs_repository_create(user_config, tx));
    }

    pub fn with_handler<F, Fut>(&mut self, username: &Username, f: F)
    where
        F: FnOnce(RepositoryHandler) -> Fut + Send + 'static,
        Fut: Future<Output = Vec<Message>> + Send,
    {
        let state = self.repos.get(username).unwrap().clone();

        self.task_manager.spawn(async move {
            match get_handler(state).await {
                Ok(handler) => f(handler).await,
                Err(()) => vec![],
            }
        });
    }
}

#[derive(Clone)]
enum RepositoryState {
    Loading,
    Loaded(RepositoryHandler),
    Error(String),
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
