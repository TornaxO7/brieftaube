use crate::backend::{
    Backend, LoadingRole, MailBodyType, MailDataHtmlBody, MailDataTextBody, MailId, types::Loadable,
};
use tokio::sync::watch;
use tracing::debug;

impl Backend {
    /// Panics if the mail hasn't been fetched before.
    pub async fn get_or_request_mail_body(
        &self,
        id: &MailId,
        ty: MailBodyType,
    ) -> Result<String, jmap_client::Error> {
        let role = {
            let mut store = self.store.lock().unwrap();
            let mail = store.mails.get_or_insert_mut(id).loaded_mut().unwrap();

            match mail.get_body(ty) {
                Loadable::Loaded(body) => return Ok(body.to_string()),
                Loadable::Requested { notifier } => LoadingRole::Wait(notifier.clone()),
                Loadable::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    mail.init_body(ty, rx);
                    LoadingRole::Request(tx)
                }
            }
        };

        match role {
            LoadingRole::Wait(mut notifier) => {
                notifier.changed().await.unwrap();
                let mut store = self.store.lock().unwrap();
                let mail = store.mails.get_or_insert_mut(id).loaded_mut().unwrap();
                Ok(mail.get_body(ty).loaded().unwrap().to_string())
            }
            LoadingRole::Request(sender) => {
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
                        let mail = store.mails.get_or_insert_mut(&id).loaded_mut().unwrap();
                        mail.text_body = Loadable::Loaded(body.clone());
                        body.0
                    }
                    MailBodyType::Html => {
                        debug!("Setting html body");
                        let body = MailDataHtmlBody::new(&mail).unwrap();
                        let mail = store.mails.get_or_insert_mut(&id).loaded_mut().unwrap();
                        mail.html_body = Loadable::Loaded(body.clone());
                        body.0
                    }
                };

                let _ = sender.send(());

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
