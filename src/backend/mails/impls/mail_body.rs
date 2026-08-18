use crate::backend::{
    Backend, MailBodyType, MailDataHtmlBody, MailDataTextBody, MailId, types::RemoteData,
};
use tokio::sync::watch;
use tracing::debug;

enum Next {
    Wait(watch::Receiver<()>),
    Request(watch::Sender<()>),
}

impl Backend {
    pub async fn get_or_request_mail_body(
        &self,
        id: &MailId,
        ty: MailBodyType,
    ) -> Result<String, jmap_client::Error> {
        let next = {
            let mut store = self.store.lock().unwrap();
            let mail = store.mails.get_mut(id).expect("Mail exists");

            match mail.get_body(ty) {
                RemoteData::Loaded(body) => return Ok(body.to_string()),
                RemoteData::Requested { notifier } => Next::Wait(notifier.clone()),
                RemoteData::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    mail.init_body(ty, rx);
                    Next::Request(tx)
                }
            }
        };

        match next {
            Next::Wait(mut receiver) => {
                receiver.changed().await.unwrap();
                let store = self.store.lock().unwrap();
                let mail = store.mails.get(id).expect("Mail exists");
                Ok(mail.get_body(ty).loaded().unwrap().to_string())
            }
            Next::Request(sender) => {
                let mut response = {
                    let mut request = self.client.build();
                    let get_mail = request.get_email().ids(Some([&id.0]));
                    match ty {
                        MailBodyType::Text => get_mail.arguments().fetch_text_body_values(true),
                        MailBodyType::Html => get_mail.arguments().fetch_html_body_values(true),
                    };

                    request.send_get_email().await?
                };

                let mail = response.take_list()[0].clone();

                let mut store = self.store.lock().unwrap();
                store.mails.set_state(response.take_state());
                let body = match ty {
                    MailBodyType::Text => {
                        debug!("Setting text body");
                        let body = MailDataTextBody::new(&mail).unwrap();
                        let mail = store.mails.get_mut(&id).unwrap();
                        mail.text_body = RemoteData::Loaded(body.clone());
                        body.0
                    }
                    MailBodyType::Html => {
                        debug!("Setting html body");
                        let body = MailDataHtmlBody::new(&mail).unwrap();
                        let mail = store.mails.get_mut(&id).unwrap();
                        mail.html_body = RemoteData::Loaded(body.clone());
                        body.0
                    }
                };

                sender.send(()).unwrap();

                Ok(body)
            }
        }
    }

    pub async fn prefetch_mail_body(
        &self,
        id: &MailId,
        ty: MailBodyType,
    ) -> Result<(), jmap_client::Error> {
        self.get_or_request_mail_body(id, ty).await?;
        Ok(())
    }
}
