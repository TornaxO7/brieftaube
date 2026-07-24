mod cache;
mod error;
pub mod types;

use cache::Cache;
use error::MailboxValidationError;
use jmap_client::{
    URI,
    client::Client,
    core::{error::MethodErrorType, session::Capabilities, set::SetObject},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::oneshot,
    task::{JoinError, JoinHandle},
};
use tracing::error;
use types::{MailboxData, MailboxId, MailboxNew, MailboxUpdate, MailboxValidate};

pub struct MailboxBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    tasks: Mutex<VecDeque<JoinHandle<()>>>,
}

impl MailboxBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
            tasks: Mutex::new(VecDeque::with_capacity(16)),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.lock().unwrap().is_empty()
    }

    pub async fn has_changed(&self) -> Option<Result<(), JoinError>> {
        let mut guard = self.tasks.lock().unwrap();
        let task = guard.front_mut().unwrap();
        Some(task.await)
    }

    pub fn pop_task(&self) {
        self.tasks
            .lock()
            .unwrap()
            .pop_front()
            .expect("There are tasks.");
    }

    pub fn cache_is_initialised(&self) -> bool {
        !self.cache.lock().unwrap().is_empty()
    }
}

// methods which also communicate with the server
impl MailboxBackend {
    pub fn request_mailboxes(&self) {
        if self.cache_is_initialised() {
            // TODO: Request `mailbox/changes`
            return;
        }

        let client = self.client.clone();
        let cache = self.cache.clone();

        self.tasks
            .lock()
            .unwrap()
            .push_back(tokio::spawn(mailbox_get(client, cache)));
    }

    pub fn remove_mailboxes(&self, ids: Vec<MailboxId>) {
        if !self.cache_is_initialised() || ids.is_empty() {
            return;
        }

        let (tx, rx) = oneshot::channel();

        let client = self.client.clone();
        let cache = self.cache.clone();
        let mut tasks = self.tasks.lock().unwrap();

        tasks.push_back(tokio::spawn(mailbox_remove(
            client.clone(),
            cache.clone(),
            ids,
            tx,
        )));

        tasks.push_back(tokio::spawn(flush_cache_and_mailbox_get(client, cache, rx)));
    }

    pub fn update_mailboxes(&self, mailboxes: Vec<MailboxUpdate>) {
        if !self.cache_is_initialised() || mailboxes.is_empty() {
            return;
        }

        if let Err(errors) = self.validate_mailboxes(&mailboxes) {
            for error in errors {
                error!("Can't update mailbox: {}", error);
            }
            return;
        }

        let (tx, rx) = oneshot::channel();
        let cache = self.cache.clone();
        let client = self.client.clone();
        let mut tasks = self.tasks.lock().unwrap();

        tasks.push_back(tokio::spawn(mailbox_update(
            client.clone(),
            cache.clone(),
            mailboxes,
            tx,
        )));

        tasks.push_back(tokio::spawn(flush_cache_and_mailbox_get(client, cache, rx)));
    }

    pub fn create_mailboxes(&self, mailboxes: Vec<MailboxNew>) {
        if !self.cache_is_initialised() || mailboxes.is_empty() {
            return;
        }

        {
            if let Err(errors) = self.validate_mailboxes(&mailboxes) {
                for error in errors {
                    error!("Can't create new mailbox: {}", error);
                }
                return;
            }
        }

        let (tx, rx) = oneshot::channel();
        let client = self.client.clone();
        let cache = self.cache.clone();
        let mut tasks = self.tasks.lock().unwrap();

        tasks.push_back(tokio::spawn(mailbox_create(
            client.clone(),
            cache.clone(),
            mailboxes,
            tx,
        )));

        tasks.push_back(tokio::spawn(flush_cache_and_mailbox_get(client, cache, rx)));
    }

    pub fn mail_capability(&self) -> jmap_client::email::MailCapabilities {
        let id = self.client.default_account_id();

        match self
            .client
            .session()
            .account(id)
            .unwrap()
            .capability(URI::Mail.as_ref())
            .unwrap()
            .clone()
        {
            Capabilities::Mail(cap) => cap,
            _ => unreachable!(),
        }
    }
}

// helpers
impl MailboxBackend {
    fn validate_mailboxes<'a, M>(
        &self,
        mailboxes: &'a [M],
    ) -> Result<(), Vec<MailboxValidationError>>
    where
        &'a M: Into<MailboxValidate>,
    {
        let cache = self.cache.lock().unwrap();
        let caps = self.mail_capability();
        let mut errors = Vec::with_capacity(mailboxes.len());

        for mailbox in mailboxes {
            let MailboxValidate {
                name,
                role: _,
                sort_order: _,
                parent_id,
            } = mailbox.into();

            if let Some(name) = name.as_ref() {
                let min = 1;
                let max = caps.max_size_mailbox_name();

                if !(min < name.len() && name.len() <= max) {
                    errors.push(MailboxValidationError::NameTooLong { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref() {
                let max = caps.max_mailbox_depth();
                if cache.depth_of(parent_id) + 1 > max {
                    errors.push(MailboxValidationError::MaxDepthExceeded { max });
                }
            }

            if let Some(parent_id) = parent_id.as_ref()
                && let Some(name) = name.as_ref()
            {
                if cache.contains_mailbox_name(&parent_id, &name) {
                    errors.push(MailboxValidationError::DuplicateName {
                        name: name.to_string(),
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// methods for `state.rs`
impl MailboxBackend {
    pub fn get
}

// start a `Mailbox/get` request
async fn mailbox_get(client: Arc<Client>, cache: Arc<Mutex<Cache>>) {
    let mut response = {
        let mut request = client.build();
        request
            .get_mailbox()
            .ids::<[_; 1], String>(None::<[_; 1]>)
            .properties(MailboxData::PROPERTIES);

        match request.send_get_mailbox().await {
            Ok(r) => r,
            Err(err) => {
                error!("Couldn't request mailboxes from server: {err}");
                return;
            }
        }
    };

    let mut cache = cache.lock().unwrap();
    cache.flush();
    for mailbox in response.take_list() {
        let data = MailboxData::from(mailbox);
        cache.add(data);
    }

    cache.set_state(response.take_state());
}

async fn mailbox_remove(
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    ids: Vec<MailboxId>,
    tx: oneshot::Sender<Result<(), MethodErrorType>>,
) {
    let mut response = {
        let current_state = {
            let cache = cache.lock().unwrap();
            cache.get_state()
        };

        let mut request = client.build();
        let set_mailbox = request.set_mailbox();
        set_mailbox.destroy(&ids).if_in_state(current_state);
        set_mailbox.arguments().on_destroy_remove_emails(false);

        match request.send_set_mailbox().await {
            Ok(r) => r,
            Err(err) => {
                error!("Couldn't request server to destroy mailboxes: {err}");

                match err {
                    jmap_client::Error::Method(method) => match method.p_type {
                        MethodErrorType::StateMismatch => {
                            let _ = tx.send(Err(MethodErrorType::StateMismatch));
                        }
                        _ => {}
                    },
                    _ => {}
                }
                return;
            }
        }
    };

    let _ = tx.send(Ok(()));

    let mut cache = cache.lock().unwrap();
    cache.set_state(response.take_new_state());
    for id in ids.into_iter() {
        match response.destroyed(&id) {
            Ok(()) => {
                cache.remove(id);
            }
            Err(err) => match cache.get_mailbox(&id) {
                Some(mailbox) => {
                    let name = mailbox.name.clone();
                    error!("Couldn't destroy the mailbox '{name}': {err}");
                }
                None => {
                    error!("Couldn't destroy mailbox:\n{err}");
                }
            },
        }
    }
}

async fn mailbox_update(
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    mailboxes: Vec<MailboxUpdate>,
    tx: oneshot::Sender<Result<(), MethodErrorType>>,
) {
    let mut response = {
        let current_state = {
            let cache = cache.lock().unwrap();
            cache.get_state()
        };

        let mut request = client.build();
        let set_mailbox = request.set_mailbox().if_in_state(current_state);

        for mailbox in mailboxes.iter() {
            let u = set_mailbox.update(&mailbox.id);
            if let Some(name) = &mailbox.name {
                u.name(name);
            }

            if let Some(role) = mailbox.role.clone() {
                u.role(role);
            }

            if let Some(sort_order) = mailbox.sort_order.clone() {
                u.sort_order(sort_order);
            }

            if let Some(parent_id) = mailbox.parent_id.clone() {
                u.parent_id(parent_id);
            }
        }

        match request.send_set_mailbox().await {
            Ok(r) => r,
            Err(err) => {
                error!("Couldn't request server to update the mailboxes:\n{err}");

                match err {
                    jmap_client::Error::Method(method) => match method.p_type {
                        MethodErrorType::StateMismatch => {
                            let _ = tx.send(Err(MethodErrorType::StateMismatch));
                        }
                        _ => {}
                    },
                    _ => {}
                }

                return;
            }
        }
    };

    let _ = tx.send(Ok(()));

    let mut cache = cache.lock().unwrap();
    cache.set_state(response.take_new_state());

    for mailbox in mailboxes {
        match response.updated(&mailbox.id) {
            Ok(_) => {
                cache.update(mailbox);
            }
            Err(err) => match cache.get_mailbox(&mailbox.id) {
                Some(mailbox) => {
                    let name = mailbox.name.clone();
                    error!("Couldn't update the mailbox of '{name}':\n{err}");
                }
                None => {
                    error!("Couldn't update a mailbox:\n{err}");
                }
            },
        };
    }
}

async fn mailbox_create(
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    mailboxes: Vec<MailboxNew>,
    tx: oneshot::Sender<Result<(), MethodErrorType>>,
) {
    let (mut response, tmp_ids) = {
        let current_state = {
            let cache = cache.lock().unwrap();
            cache.get_state()
        };

        let mut tmp_ids = Vec::with_capacity(mailboxes.len());
        let mut request = client.build();
        let set_mailbox = request.set_mailbox().if_in_state(current_state);

        for mailbox in mailboxes.iter() {
            let c = set_mailbox.create();
            c.name(&mailbox.name);
            c.parent_id(mailbox.parent_id.clone());

            if let Some(role) = mailbox.role.clone() {
                c.role(role);
            }

            if let Some(sort_order) = mailbox.sort_order {
                c.sort_order(sort_order);
            }

            tmp_ids.push(c.create_id().unwrap());
        }

        match request.send_set_mailbox().await {
            Ok(r) => (r, tmp_ids),
            Err(err) => {
                error!("Couldn't request server to update the mailboxes:\n{err}");

                match err {
                    jmap_client::Error::Method(method) => match method.p_type {
                        MethodErrorType::StateMismatch => {
                            let _ = tx.send(Err(MethodErrorType::StateMismatch));
                        }
                        _ => {}
                    },
                    _ => {}
                }

                return;
            }
        }
    };

    let _ = tx.send(Ok(()));

    let mut cache = cache.lock().unwrap();
    cache.set_state(response.take_new_state());

    for (mailbox, tmp_id) in mailboxes.into_iter().zip(tmp_ids.into_iter()) {
        match response.created(&tmp_id) {
            Ok(mut server) => {
                let id = server.take_id();
                let name = server
                    .name()
                    .map(ToString::to_string)
                    .unwrap_or(mailbox.name);
                let role = server.role();
                let sort_order = server.sort_order();
                let parent_id = mailbox.parent_id;
                let unread_mails = server.unread_emails();

                let mailbox = MailboxData {
                    id,
                    name,
                    role,
                    sort_order,
                    parent_id,
                    unread_mails,
                };

                cache.add(mailbox);
            }
            Err(err) => {
                error!("Couldn't create mailbox '{}': {err}", mailbox.name);
                return;
            }
        };
    }
}

async fn flush_cache_and_mailbox_get(
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
    rx: oneshot::Receiver<Result<(), MethodErrorType>>,
) {
    let recv = match rx.await {
        Ok(r) => r,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let Err(error) = recv else {
        return;
    };

    match error {
        MethodErrorType::StateMismatch => mailbox_get(client, cache).await,
        _ => {}
    };
}
