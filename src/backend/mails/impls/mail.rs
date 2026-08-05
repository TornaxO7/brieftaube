use crate::backend::{Backend, MailData, MailId, mails::Store, task_manager::TaskId};
use jmap_client::{
    Set,
    core::{get::GetRequest, response::EmailGetResponse},
    email::Email,
};
use tracing::error;

impl Backend {
    pub fn get_or_request_mail(&self, id: &MailId) -> Option<MailData> {
        let mail = self.get_mail(id);

        if mail.is_none() {
            self.request_mails(&[id.to_owned()]);
        }

        mail
    }

    pub fn get_mail(&self, id: &MailId) -> Option<MailData> {
        let store = self.store.lock().unwrap();
        store.mails.get(id).cloned()
    }

    pub fn request_mails(&self, ids: &[MailId]) {
        let client = self.client.clone();
        let ids = ids.to_owned();
        let store = self.store.clone();

        self.task_manager.spawn(TaskId::RequestMails, async move {
            let mut request = client.build();

            request_get_mails(request.get_email(), &ids);

            let response = match request.send_get_email().await {
                Ok(r) => r,
                Err(err) => {
                    error!("Couldn't send `Email/get` request to server:\n{err}");
                    return;
                }
            };

            let mut store = store.lock().unwrap();
            handle_get_mails(&mut store.mails, response);
        });
    }
}

pub fn request_get_mails<'a>(request: &mut GetRequest<Email<Set>>, ids: &[MailId]) {
    request
        .properties(MailData::PROPERTIES)
        .ids(Some(ids.iter().map(|id| &id.0)));
}

pub fn handle_get_mails(store: &mut Store, mut response: EmailGetResponse) {
    store.set_state(response.take_state());
    for mail in response.take_list() {
        store.add(MailData::new(mail));
    }
}
