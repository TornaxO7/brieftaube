use crate::backend::{Backend, FetchRole, MailboxData, ParentMailboxId, types::RemoteData};
use tokio::sync::watch;

impl Backend {
    pub async fn get_mailbox_children(
        &self,
        parent: ParentMailboxId,
    ) -> Result<Vec<MailboxData>, jmap_client::Error> {
        let role = {
            let mut store = self.store.lock().unwrap();
            let ids = store.mailbox.get_children_ids_mut(&parent);

            match ids {
                RemoteData::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    *ids = RemoteData::Requested { notifier: rx };
                    FetchRole::Request(tx)
                }
                RemoteData::Requested { notifier } => FetchRole::Wait(notifier.clone()),
                RemoteData::Loaded(ids) => {
                    let ids = ids.clone();

                    let datas = ids
                        .into_iter()
                        .map(|id| store.mailbox.get_data(&id).clone())
                        .collect();

                    return Ok(datas);
                }
            }
        };

        match role {
            FetchRole::Wait(mut notifier) => {
                notifier.changed().await.unwrap();

                let mut store = self.store.lock().unwrap();
                let ids = store
                    .mailbox
                    .get_children_ids(&parent)
                    .loaded()
                    .unwrap()
                    .clone();

                let datas = ids
                    .into_iter()
                    .map(|id| store.mailbox.get_data(&id).clone())
                    .collect();

                Ok(datas)
            }
            FetchRole::Request(notifier) => {
                let mut response = {
                    let mut request = self.client.build();

                    let query_result = request
                        .query_mailbox()
                        .filter(jmap_client::mailbox::query::Filter::ParentId {
                            value: parent.clone().map(|id| id.0),
                        })
                        .result_reference();

                    request
                        .get_mailbox()
                        .ids_ref(query_result)
                        .properties(MailboxData::PROPERTIES);

                    request.send().await?
                };

                let mut mailbox_get_response = response
                    .pop_method_response()
                    .unwrap()
                    .unwrap_get_mailbox()
                    .unwrap();
                response.pop_method_response().expect("Query result");

                let child_mailboxes: Vec<MailboxData> = mailbox_get_response
                    .take_list()
                    .into_iter()
                    .map(MailboxData::from)
                    .collect();

                let mut store = self.store.lock().unwrap();
                store
                    .mailbox
                    .set_children_query_state(parent.clone(), mailbox_get_response.take_state());

                store.mailbox.add_children(&parent, child_mailboxes.clone());

                let _ = notifier.send(());

                Ok(child_mailboxes)
            }
        }
    }
}
