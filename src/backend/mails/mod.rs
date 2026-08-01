mod cache;
pub mod types;

use crate::backend::{
    Backend,
    mails::types::{
        MailBodyType, MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailUpdate,
    },
    task_manager::TaskId,
    threads::types::ThreadId,
};
use cache::Cache;
use jmap_client::{client::Client, core::response::EmailGetResponse};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, instrument, warn};
use types::MailId;

pub struct MailsBackend {
    client: Arc<Client>,
    cache: Arc<Mutex<Cache>>,
}

impl MailsBackend {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }
}

/// local getter and setter
impl MailsBackend {
    pub fn get_data(&self, id: &MailId) -> Option<MailData> {
        let cache = self.cache.lock().unwrap();
        cache.get_mail(id).cloned()
    }

    pub fn get_datas(&self, ids: &[MailId]) -> Option<Vec<MailData>> {
        let cache = self.cache.lock().unwrap();

        ids.iter().map(|id| cache.get_mail(id).cloned()).collect()
    }

    pub fn handle_get_response(&self, mut response: EmailGetResponse) {
        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());
        for mail in response.take_list() {
            cache.add(MailData::new(mail));
        }
    }
}

/// Request methods
impl MailsBackend {
    async fn request_mails(&self, ids: &[MailId]) -> Result<(), jmap_client::Error> {
        let response = {
            let mut request = self.client.build();

            request
                .get_email()
                .properties(MailData::PROPERTIES)
                .ids(Some(ids.iter().map(|id| &id.0)));

            request.send_get_email().await?
        };

        self.handle_get_response(response);
        Ok(())
    }

    async fn request_update_mails(
        &self,
        updates: Vec<MailUpdate>,
    ) -> Result<(), jmap_client::Error> {
        let mut response = {
            let current_state = {
                let cache = self.cache.lock().unwrap();
                cache.get_state()
            };

            let mut request = self.client.build();
            let set_mail_request = request.set_email().if_in_state(current_state);

            for update in updates.iter() {
                if !update.is_empty() {
                    let mail_to_update = set_mail_request.update(&update.id.0);

                    if let Some(keywords) = update.patch_keywords.as_ref() {
                        for (keyword, set) in keywords {
                            mail_to_update.keyword(keyword.to_string().as_str(), *set);
                        }
                    }

                    if let Some(new_mailboxes) = update.mailbox_ids.as_ref() {
                        for (mailbox_id, set) in new_mailboxes {
                            mail_to_update.mailbox_id(&mailbox_id.0, *set);
                        }
                    }
                }
            }

            request.send_set_email().await?
        };

        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_new_state());

        for update in updates {
            match response.updated(&update.id.0)? {
                None => {}
                Some(huh) => warn!(
                    "The server sent an unexpected response mail:\n{huh:?}\nCould you please create an issue? :>"
                ),
            }
            cache.update(update);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn request_body_type(
        &self,
        id: &MailId,
        body_type: MailBodyType,
    ) -> Result<(), jmap_client::Error> {
        let mut response = {
            let mut request = self.client.build();
            let get_mail = request.get_email().ids(Some([&id.0]));
            match body_type {
                MailBodyType::Text => get_mail.arguments().fetch_text_body_values(true),
                MailBodyType::Html => get_mail.arguments().fetch_html_body_values(true),
            };

            request.send_get_email().await?
        };

        let mail = response.take_list()[0].clone();

        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());
        match body_type {
            MailBodyType::Text => {
                debug!("Setting text body");
                let body = MailDataTextBody::new(&mail);
                cache.set_text_body(&id, body);
            }
            MailBodyType::Html => {
                debug!("Setting html body");
                let body = MailDataHtmlBody::new(&mail);
                cache.set_html_body(&id, body);
            }
        }

        Ok(())
    }

    async fn request_attachments(&self, id: MailId) -> Result<(), jmap_client::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some([&id.0]))
                .properties([jmap_client::email::Property::Attachments]);

            request.send_get_email().await?
        };

        let mail = response.take_list()[0].clone();
        let attachments: Vec<MailDataAttachment> = mail
            .attachments()
            .unwrap()
            .iter()
            .map(MailDataAttachment::from)
            .collect();

        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());
        cache.set_attachments(&id, attachments);

        Ok(())
    }
}

impl Backend {
    pub fn mail_get_data(&self, id: &MailId) -> Option<MailData> {
        self.mails.get_data(id)
    }

    pub fn mail_get_or_request_thread_mails(&self, id: &ThreadId) -> Option<Vec<MailData>> {
        let thread_mail_ids = self
            .threads
            .get(id)
            .expect("Thread has already been requested.");

        self.mails.get_datas(&thread_mail_ids).or_else(|| {
            let mails = self.mails.clone();

            self.task_manager.spawn(TaskId::GetThreadMails, async move {
                if let Err(err) = mails.request_mails(&thread_mail_ids).await {
                    error!("Couldn't request mails of thread:\n{err}");
                }
            });
            None
        })
    }

    pub fn mail_request_body_type(&self, id: &MailId, body_type: MailBodyType) {
        let already_cached = {
            let mail = self.mails.get_data(id).unwrap();
            match body_type {
                MailBodyType::Text => mail.text_body.is_some(),
                MailBodyType::Html => mail.html_body.is_some(),
            }
        };

        if !already_cached {
            let id = id.clone();
            let mails = self.mails.clone();
            self.task_manager.spawn(TaskId::FetchBodyType, async move {
                if let Err(err) = mails.request_body_type(&id, body_type).await {
                    error!("Couldn't request body of mail:\n{err}");
                }
            });
        }
    }

    pub fn mail_request_attachments(&self, id: &MailId) {
        let attachments_already_fetched = {
            let mail = self.mails.get_data(id).unwrap();
            mail.attachments.is_some()
        };

        if !attachments_already_fetched {
            let id = id.clone();
            let mails = self.mails.clone();

            self.task_manager
                .spawn(TaskId::FetchMailAttachments, async move {
                    if let Err(err) = mails.request_attachments(id).await {
                        error!("Couldn't request attachments of mail:\n{err}");
                    }
                })
        }
    }

    pub fn mails_update(&self, updates: Vec<MailUpdate>) {
        let updates_are_empty = updates.iter().all(|update| update.is_empty());
        if updates.is_empty() || updates_are_empty {
            return;
        }

        let mails = self.mails.clone();
        self.task_manager.spawn(TaskId::SetMailSeen, async move {
            if let Err(err) = mails.request_update_mails(updates).await {
                error!("Couldn't send request to update mails to server:\n{err}");
            }
        });
    }
}
