use crate::backend::{Backend, MailBodyType, MailDataHtmlBody, MailDataTextBody, MailId};
use tracing::debug;

impl Backend {
    pub async fn get_or_request_mail_body(
        &self,
        id: &MailId,
        ty: MailBodyType,
    ) -> Result<String, jmap_client::Error> {
        match self.get_mail_body_type(id, ty) {
            Some(body) => Ok(body),
            None => {
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
                match ty {
                    MailBodyType::Text => {
                        debug!("Setting text body");
                        let body = MailDataTextBody::new(&mail);
                        store.mails.set_text_body(&id, body.clone());
                        Ok(body.0)
                    }
                    MailBodyType::Html => {
                        debug!("Setting html body");
                        let body = MailDataHtmlBody::new(&mail);
                        store.mails.set_html_body(&id, body.clone());
                        Ok(body.0)
                    }
                }
            }
        }
    }

    pub async fn prefetch_mail_body(
        &self,
        id: &MailId,
        ty: MailBodyType,
    ) -> Result<(), jmap_client::Error> {
        self.get_or_request_mail_body(id, ty).await.map(|_| ())
    }

    pub fn get_mail_body_type(&self, id: &MailId, ty: MailBodyType) -> Option<String> {
        let store = self.store.lock().unwrap();
        let mail = store.mails.get(id).unwrap();

        match ty {
            MailBodyType::Text => mail.text_body.as_ref().map(|text| text.0.clone()),
            MailBodyType::Html => mail.html_body.as_ref().map(|html| html.0.clone()),
        }
    }
}
