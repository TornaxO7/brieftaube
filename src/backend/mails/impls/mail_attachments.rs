use jmap_client::{
    Set,
    core::{get::GetRequest, response::EmailGetResponse},
    email::Email,
};
use tracing::error;

use crate::backend::{Backend, MailDataAttachment, MailId, mails::Store, task_manager::TaskId};

impl Backend {
    pub fn get_or_request_mail_attachments(&self, id: &MailId) -> Option<Vec<MailDataAttachment>> {
        let attachments = self.get_mail_attachments(id);

        if attachments.is_none() {
            self.request_mails_attachments(&[id.clone()]);
        }

        attachments
    }

    pub fn get_mail_attachments(&self, id: &MailId) -> Option<Vec<MailDataAttachment>> {
        let store = self.store.lock().unwrap();

        store
            .mails
            .get(id)
            .and_then(|mail| mail.attachments.clone())
    }

    pub fn request_mails_attachments(&self, ids: &[MailId]) {
        let ids = ids.to_owned();
        let store = self.store.clone();
        let client = self.client.clone();

        self.task_manager
            .spawn(TaskId::FetchMailAttachments, async move {
                let mut request = client.build();

                request_mail_attachments(request.get_email(), &ids);

                let response = match request.send_get_email().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request attachments of mail:\n{err}");
                        return;
                    }
                };

                let mut store = store.lock().unwrap();
                handle_mail_attachments(&mut store.mails, response);
            })
    }
}

fn request_mail_attachments(request: &mut GetRequest<Email<Set>>, ids: &[MailId]) {
    request
        .ids(Some(ids.iter().map(|id| &id.0)))
        .properties([jmap_client::email::Property::Attachments]);
}

fn handle_mail_attachments(store: &mut Store, mut response: EmailGetResponse) {
    store.set_state(response.take_state());

    for mail in response.take_list() {
        let attachments: Vec<MailDataAttachment> = mail
            .attachments()
            .unwrap()
            .iter()
            .map(MailDataAttachment::from)
            .collect();

        store.set_attachments(&MailId(mail.id().unwrap().to_owned()), attachments);
    }
}
