use crate::backend::{Backend, FetchRole, MailData, MailId, ThreadId, types::RemoteData};
use tokio::sync::watch;

impl Backend {
    pub async fn get_thread_mail_ids(
        &self,
        id: &ThreadId,
    ) -> Result<Vec<MailId>, jmap_client::Error> {
        let role = {
            let mut store = self.store.lock().unwrap();
            let ids = store.threads.get_mail_ids_mut(id);

            match ids {
                RemoteData::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    *ids = RemoteData::Requested { notifier: rx };
                    FetchRole::Request(tx)
                }
                RemoteData::Requested { notifier } => FetchRole::Wait(notifier.clone()),
                RemoteData::Loaded(ids) => return Ok(ids.clone()),
            }
        };

        match role {
            FetchRole::Wait(mut receiver) => {
                receiver.changed().await.unwrap();
                let mut store = self.store.lock().unwrap();
                let ids = store.threads.get_mail_ids(id).loaded().unwrap();
                Ok(ids.clone())
            }
            FetchRole::Request(_sender) => {
                unreachable!("Should be requested from the logic. Otherwise: Fill this");
            }
        }
    }

    pub async fn get_thread_mails(
        &self,
        id: &ThreadId,
    ) -> Result<Vec<MailData>, jmap_client::Error> {
        let ids = self.get_thread_mail_ids(id).await?;
        self.get_mails(&ids).await
    }
}
