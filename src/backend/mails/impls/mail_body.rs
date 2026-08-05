use crate::backend::{
    Backend, MailBodyType, MailDataHtmlBody, MailDataTextBody, MailId, mails::Store,
    task_manager::TaskId,
};
use jmap_client::{
    Set,
    core::{get::GetRequest, response::EmailGetResponse},
    email::Email,
};
use tracing::{debug, error};

impl Backend {
    pub fn get_or_request_mail_body(&self, id: &MailId, ty: MailBodyType) -> Option<String> {
        let body = self.get_mail_body_type(id, ty);

        if body.is_none() {
            self.request_mail_body(id, body_type);
        }

        body
    }

    fn get_mail_body_type(&self, id: &MailId, ty: MailBodyType) -> Option<String> {
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
            let mut request = client.build();

            request_body_type(request.get_email(), &id, body_type);

            let response = match request.send_get_email().await {
                Ok(r) => r,
                Err(err) => {
                    error!("Couldn't request body of mail:\n{err}");
                    return;
                }
            };

            let mut store = store.lock().unwrap();
            handle_body_type(&mut store.mails, &id, body_type, response);
        });
    }
}

fn request_body_type<'a>(
    request: &mut GetRequest<Email<Set>>,
    id: &MailId,
    body_type: MailBodyType,
) {
    let get_mail = request.ids(Some([&id.0]));
    match body_type {
        MailBodyType::Text => get_mail.arguments().fetch_text_body_values(true),
        MailBodyType::Html => get_mail.arguments().fetch_html_body_values(true),
    };
}

fn handle_body_type(
    store: &mut Store,
    id: &MailId,
    body_type: MailBodyType,
    mut response: EmailGetResponse,
) {
    let mail = response.take_list()[0].clone();

    store.set_state(response.take_state());
    match body_type {
        MailBodyType::Text => {
            debug!("Setting text body");
            let body = MailDataTextBody::new(&mail);
            store.set_text_body(&id, body);
        }
        MailBodyType::Html => {
            debug!("Setting html body");
            let body = MailDataHtmlBody::new(&mail);
            store.set_html_body(&id, body);
        }
    }
}
