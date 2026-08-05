use crate::backend::{Backend, MailDataAttachment, MailId, task_manager::TaskId};
use tracing::error;

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
                let mut response = {
                    let mut request = client.build();

                    request
                        .get_email()
                        .ids(Some(ids.iter().map(|id| &id.0)))
                        .properties([jmap_client::email::Property::Attachments]);

                    match request.send_get_email().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't request attachments of mail:\n{err}");
                            return;
                        }
                    }
                };

                let mut store = store.lock().unwrap();
                store.mails.set_state(response.take_state());

                for mail in response.take_list() {
                    let attachments: Vec<MailDataAttachment> = mail
                        .attachments()
                        .unwrap()
                        .iter()
                        .map(MailDataAttachment::from)
                        .collect();

                    store
                        .mails
                        .set_attachments(&MailId(mail.id().unwrap().to_owned()), attachments);
                }
            })
    }
}
