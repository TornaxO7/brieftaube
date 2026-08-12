use crate::backend::{Backend, MailData, MailId};
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
            let mut response = {
                let mut request = client.build();

                request
                    .get_email()
                    .properties(MailData::PROPERTIES)
                    .ids(Some(ids.iter().map(|id| &id.0)));

                match request.send_get_email().await {
                    Ok(r) => r,
                    Err(err) => {
                        error!("Couldn't send `Email/get` request to server:\n{err}");
                        return;
                    }
                }
            };

            let mut store = store.lock().unwrap();
            store.mails.set_state(response.take_state());
            for mail in response.take_list() {
                store.mails.add(MailData::new(mail));
            }
        });
    }
}
