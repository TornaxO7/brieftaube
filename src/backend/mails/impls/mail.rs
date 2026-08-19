use crate::backend::{Backend, MailData, MailId, types::RemoteData};
use std::collections::HashMap;
use tokio::{join, sync::watch, task::JoinSet};

impl Backend {
    pub async fn get_mail(&self, id: &MailId) -> Result<MailData, jmap_client::Error> {
        self.get_mails(&[id.clone()])
            .await
            .map(|mails| mails[0].clone())
    }

    pub async fn get_mails(&self, ids: &[MailId]) -> Result<Vec<MailData>, jmap_client::Error> {
        let mut datas: Vec<MailData> = Vec::with_capacity(ids.len());

        let mut not_requested: HashMap<MailId, watch::Sender<()>> = HashMap::new();
        let mut requested: Vec<(MailId, watch::Receiver<()>)> = Vec::new();

        {
            let mut store = self.store.lock().unwrap();

            for id in ids {
                let mail = store.mails.get_mut(id);

                match mail {
                    RemoteData::NotRequested => {
                        let (tx, rx) = watch::channel(());
                        *mail = RemoteData::Requested { notifier: rx };
                        not_requested.insert(id.clone(), tx);
                    }
                    RemoteData::Requested { notifier } => {
                        requested.push((id.clone(), notifier.clone()))
                    }
                    RemoteData::Loaded(data) => datas.push(data.clone()),
                }
            }
        }

        let (requested_data, awaiting_data) = join!(
            self.request_mails(not_requested),
            self.await_requested_mails(requested)
        );

        datas.extend(requested_data?);
        datas.extend(awaiting_data);

        Ok(datas)
    }

    async fn request_mails(
        &self,
        missing: HashMap<MailId, watch::Sender<()>>,
    ) -> Result<Vec<MailData>, jmap_client::Error> {
        if missing.is_empty() {
            return Ok(vec![]);
        }

        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .properties(MailData::PROPERTIES)
                .ids(Some(missing.iter().map(|(id, _)| &id.0)));

            request.send_get_email().await?
        };

        let mut store = self.store.lock().unwrap();
        store.mails.set_state(response.take_state());

        let fetched_mails: Vec<MailData> = response
            .take_list()
            .into_iter()
            .map(MailData::from)
            .collect();

        for fetched_mail in fetched_mails.iter() {
            let sender = missing.get(&fetched_mail.id).unwrap();
            let _notify = sender.send(());

            store.mails.add(fetched_mail.clone());
        }

        Ok(fetched_mails)
    }

    async fn await_requested_mails(
        &self,
        requested: Vec<(MailId, watch::Receiver<()>)>,
    ) -> Vec<MailData> {
        if requested.is_empty() {
            return vec![];
        }

        let mut set: JoinSet<MailId> = JoinSet::new();

        for (id, mut notifier) in requested {
            set.spawn(async move {
                notifier.changed().await.unwrap();
                id
            });
        }

        let mut store = self.store.lock().unwrap();

        set.join_all()
            .await
            .into_iter()
            .map(|id| store.mails.get(&id).loaded().unwrap().clone())
            .collect()
    }
}
