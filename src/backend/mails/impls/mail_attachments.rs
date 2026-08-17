use crate::backend::{Backend, MailDataAttachment, MailId, types::Loadable};

impl Backend {
    pub async fn get_or_request_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Loadable<Vec<MailDataAttachment>>, jmap_client::Error> {
        match self.get_mail_attachments(id) {
            Some(attachments) => Ok(attachments),
            None => {
                {
                    let mut store = self.store.lock().unwrap();
                    store.mails.init_attachments(id);
                }

                let mut response = {
                    let mut request = self.client.build();

                    request
                        .get_email()
                        .ids(Some([&id.0]))
                        .properties([jmap_client::email::Property::Attachments]);

                    request.send_get_email().await?
                };

                let mut store = self.store.lock().unwrap();
                store.mails.set_state(response.take_state());

                let mail = response.list()[0].clone();
                let attachments: Vec<MailDataAttachment> = mail
                    .attachments()
                    .unwrap()
                    .iter()
                    .map(MailDataAttachment::from)
                    .collect();

                store.mails.set_attachments(&id, attachments.clone());

                Ok(Loadable::Loaded(attachments))
            }
        }
    }

    pub async fn prefetch_mail_attachments(&self, id: &MailId) -> Result<(), jmap_client::Error> {
        self.get_or_request_mail_attachments(id).await?;
        Ok(())
    }

    fn get_mail_attachments(&self, id: &MailId) -> Option<Loadable<Vec<MailDataAttachment>>> {
        let store = self.store.lock().unwrap();

        store
            .mails
            .get(id)
            .and_then(|mail| mail.attachments.get().cloned())
    }
}
