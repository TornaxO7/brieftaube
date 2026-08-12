use crate::backend::{Backend, MailData, MailId};

impl Backend {
    pub fn get_mail(&self, id: &MailId) -> Option<MailData> {
        let store = self.store.lock().unwrap();
        store.mails.get(id).cloned()
    }

    pub async fn get_or_request_mail(&self, id: &MailId) -> Result<MailData, jmap_client::Error> {
        self.get_or_request_mails(&[id.clone()])
            .await
            .map(|mails| mails[0].clone())
    }

    pub fn get_mails(&self, ids: &[MailId]) -> Option<Vec<MailData>> {
        let store = self.store.lock().unwrap();
        ids.iter().map(|id| store.mails.get(id).cloned()).collect()
    }

    pub async fn get_or_request_mails(
        &self,
        ids: &[MailId],
    ) -> Result<Vec<MailData>, jmap_client::Error> {
        match self.get_mails(ids) {
            Some(mails) => Ok(mails),
            None => {
                let mut response = {
                    let mut request = self.client.build();

                    request
                        .get_email()
                        .properties(MailData::PROPERTIES)
                        .ids(Some(ids.iter().map(|id| &id.0)));

                    request.send_get_email().await?
                };

                let mut store = self.store.lock().unwrap();
                store.mails.set_state(response.take_state());

                let mails: Vec<MailData> = response
                    .take_list()
                    .into_iter()
                    .map(MailData::from)
                    .collect();

                for mail in mails.iter() {
                    store.mails.add(mail.clone());
                }

                Ok(mails)
            }
        }
    }
}
