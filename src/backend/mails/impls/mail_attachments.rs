use tokio::sync::watch;

use crate::backend::{Backend, MailDataAttachment, MailId, types::RemoteData};

enum Next {
    Wait(watch::Receiver<()>),
    Request(watch::Sender<()>),
}

impl Backend {
    pub async fn get_or_request_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Vec<MailDataAttachment>, jmap_client::Error> {
        let next = {
            let mut store = self.store.lock().unwrap();
            let mail = store.mails.get_mut(id).expect("Mail exists");

            match &mail.attachments {
                RemoteData::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    mail.init_attachments(rx);
                    Next::Request(tx)
                }
                RemoteData::Requested { notifier } => Next::Wait(notifier.clone()),
                RemoteData::Loaded(attachments) => return Ok(attachments.clone()),
            }
        };

        match next {
            Next::Wait(mut receiver) => {
                receiver.changed().await.unwrap();
                let store = self.store.lock().unwrap();
                let mail = store.mails.get(id).unwrap();
                Ok(mail.attachments.loaded().unwrap().clone())
            }
            Next::Request(sender) => {
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

                let remote_mail = response.list()[0].clone();
                let attachments: Vec<MailDataAttachment> = remote_mail
                    .attachments()
                    .unwrap()
                    .iter()
                    .map(MailDataAttachment::from)
                    .collect();

                let mail = store.mails.get_mut(&id).unwrap();
                mail.attachments = RemoteData::Loaded(attachments.clone());
                sender.send(()).unwrap();

                Ok(attachments)
            }
        }
    }

    pub async fn prefetch_mail_attachments(&self, id: &MailId) -> Result<(), jmap_client::Error> {
        self.get_or_request_mail_attachments(id).await?;
        Ok(())
    }
}
