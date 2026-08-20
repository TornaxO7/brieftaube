use crate::backend::{
    Backend, LoadingRole, MailboxData, MailboxId, ParentMailboxId, types::Loadable,
};
use tokio::sync::watch;

impl Backend {
    pub async fn get_mailbox_children(
        &self,
        parent: ParentMailboxId,
    ) -> Result<Vec<MailboxId>, jmap_client::Error> {
        let role = {
            let mut store = self.store.lock().unwrap();
            let ids = store.mailbox.get_children_ids_mut(&parent);

            match ids {
                Loadable::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    *ids = Loadable::Requested { notifier: rx };
                    LoadingRole::Request(tx)
                }
                Loadable::Requested { notifier } => LoadingRole::Wait(notifier.clone()),
                Loadable::Loaded(ids) => return Ok(ids.clone()),
            }
        };

        match role {
            LoadingRole::Wait(mut notifier) => {
                notifier.changed().await.unwrap();

                let mut store = self.store.lock().unwrap();
                let ids = store
                    .mailbox
                    .get_children_ids(&parent)
                    .loaded()
                    .unwrap()
                    .clone();

                Ok(ids)
            }
            LoadingRole::Request(notifier) => {
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

                let ids = child_mailboxes.iter().map(|data| data.id.clone()).collect();

                let mut store = self.store.lock().unwrap();
                store
                    .mailbox
                    .set_children_query_state(parent.clone(), mailbox_get_response.take_state());

                store.mailbox.add_children(&parent, child_mailboxes.clone());

                let _ = notifier.send(());

                Ok(ids)
            }
        }
    }
}
