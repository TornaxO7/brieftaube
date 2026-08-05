use crate::backend::{
    Backend, MailBodyType, MailDataHtmlBody, MailDataTextBody, MailId, task_manager::TaskId,
};
use tracing::{debug, error};

impl Backend {
    pub fn get_or_request_mail_body(&self, id: &MailId, ty: MailBodyType) -> Option<String> {
        let body = self.get_mail_body_type(id, ty);

        if body.is_none() {
            self.request_mail_body(id, ty);
        }

        body
    }

    pub fn prefetch_mail_body(&self, id: &MailId, ty: MailBodyType) {
        self.get_or_request_mail_body(id, ty);
    }

    pub fn get_mail_body_type(&self, id: &MailId, ty: MailBodyType) -> Option<String> {
        let store = self.store.lock().unwrap();
        let mail = store.mails.get(id).unwrap();

        match ty {
            MailBodyType::Text => mail.text_body.as_ref().map(|text| text.0.clone()),
            MailBodyType::Html => mail.html_body.as_ref().map(|html| html.0.clone()),
        }
    }

    fn request_mail_body(&self, id: &MailId, body_type: MailBodyType) {
        let client = self.client.clone();
        let id = id.clone();
        let store = self.store.clone();

        self.task_manager.spawn(TaskId::FetchBodyType, async move {
            let mut response = {
                let mut request = client.build();
                let get_mail = request.get_email().ids(Some([&id.0]));
                match body_type {
                    MailBodyType::Text => get_mail.arguments().fetch_text_body_values(true),
                    MailBodyType::Html => get_mail.arguments().fetch_html_body_values(true),
                };

                match request.send_get_email().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't request body of mail:\n{err}");
                        return;
                    }
                }
            };

            let mail = response.take_list()[0].clone();

            let mut store = store.lock().unwrap();
            store.mails.set_state(response.take_state());
            match body_type {
                MailBodyType::Text => {
                    debug!("Setting text body");
                    let body = MailDataTextBody::new(&mail);
                    store.mails.set_text_body(&id, body);
                }
                MailBodyType::Html => {
                    debug!("Setting html body");
                    let body = MailDataHtmlBody::new(&mail);
                    store.mails.set_html_body(&id, body);
                }
            }
        });
    }
}
