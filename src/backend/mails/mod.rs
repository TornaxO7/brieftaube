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
use jmap_client::{
    Set,
    core::{
        get::GetRequest,
        request::Request,
        response::{Response, TaggedMethodResponse},
    },
    email::Email,
};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, warn};
use types::MailId;

pub struct MailsBackend {
    cache: Arc<Mutex<Cache>>,
}

impl MailsBackend {
    pub fn new() -> Self {
        Self {
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
}

impl MailsBackend {
    pub fn request_get_mail<'a>(&self, request: &mut GetRequest<Email<Set>>, ids: &[MailId]) {
        request
            .properties(MailData::PROPERTIES)
            .ids(Some(ids.iter().map(|id| &id.0)));
    }

    pub fn handle_get_mail(&self, response: &mut Response<TaggedMethodResponse>) {
        let mut response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();

        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_state());
        for mail in response.take_list() {
            cache.add(MailData::new(mail));
        }
    }
}

impl MailsBackend {
    pub fn request_update_mails<'a>(&self, request: &mut Request<'a>, updates: &[MailUpdate]) {
        let current_state = {
            let cache = self.cache.lock().unwrap();
            cache.get_state()
        };
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
    }

    pub fn handle_update_mails(
        &self,
        response: &mut Response<TaggedMethodResponse>,
        updates: Vec<MailUpdate>,
    ) {
        let mut response = response
            .pop_method_response()
            .unwrap()
            .unwrap_set_email()
            .unwrap();

        let mut cache = self.cache.lock().unwrap();
        cache.set_state(response.take_new_state());

        for update in updates {
            match response.updated(&update.id.0) {
                Ok(None) => {}
                Ok(Some(huh)) => warn!(
                    "The server sent an unexpected response mail:\n{huh:?}\nCould you please create an issue? :>"
                ),
                Err(err) => {
                    error!("Couldn't update mail:\n{err}");
                    continue;
                }
            }
            cache.update(update);
        }
    }
}

impl MailsBackend {
    pub fn request_body_type<'a>(
        &self,
        request: &mut Request<'a>,
        id: &MailId,
        body_type: MailBodyType,
    ) {
        let get_mail = request.get_email().ids(Some([&id.0]));
        match body_type {
            MailBodyType::Text => get_mail.arguments().fetch_text_body_values(true),
            MailBodyType::Html => get_mail.arguments().fetch_html_body_values(true),
        };
    }

    pub fn handle_body_type(
        &self,
        response: &mut Response<TaggedMethodResponse>,
        id: &MailId,
        body_type: MailBodyType,
    ) {
        let mut response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();

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
    }
}

impl MailsBackend {
    pub fn request_attachments<'a>(&self, request: &mut Request<'a>, id: &MailId) {
        request
            .get_email()
            .ids(Some([&id.0]))
            .properties([jmap_client::email::Property::Attachments]);
    }

    pub fn handle_attachments(&self, response: &mut Response<TaggedMethodResponse>, id: MailId) {
        let mut response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();

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
            let client = self.client.clone();
            let mails = self.mails.clone();

            self.task_manager.spawn(TaskId::GetThreadMails, async move {
                let mut request = client.build();
                mails.request_get_mail(request.get_email(), &thread_mail_ids);

                let mut response = match request.send().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request mails of thread:\n{err}");
                        return;
                    }
                };

                mails.handle_get_mail(&mut response);
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
            let client = self.client.clone();
            let id = id.clone();
            let mails = self.mails.clone();
            self.task_manager.spawn(TaskId::FetchBodyType, async move {
                let mut request = client.build();
                mails.request_body_type(&mut request, &id, body_type);

                if let Err(err) = request.send_get_email().await {
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
            let client = self.client.clone();

            self.task_manager
                .spawn(TaskId::FetchMailAttachments, async move {
                    let mut request = client.build();

                    mails.request_attachments(&mut request, &id);

                    let mut response = match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request attachments of mail:\n{err}");
                            return;
                        }
                    };

                    mails.handle_attachments(&mut response, id);
                })
        }
    }

    pub fn mails_update(&self, updates: Vec<MailUpdate>) {
        let updates_are_empty = updates.iter().all(|update| update.is_empty());
        if updates.is_empty() || updates_are_empty {
            return;
        }

        let client = self.client.clone();
        let mails = self.mails.clone();
        self.task_manager.spawn(TaskId::SetMailSeen, async move {
            let mut request = client.build();

            mails.request_update_mails(&mut request, &updates);

            let mut response = match request.send().await {
                Ok(r) => r,
                Err(err) => {
                    error!("Couldn't send request to update mails to server:\n{err}");
                    return;
                }
            };

            mails.handle_update_mails(&mut response, updates);
        });
    }
}
