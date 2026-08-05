use crate::backend::{Backend, MailboxData, MailboxId, ParentMailboxId};
use tracing::error;

impl Backend {
    pub fn get_or_request_mailbox_children(
        &self,
        parent: ParentMailboxId,
    ) -> Option<Vec<MailboxId>> {
        let children = self.get_mailbox_children(parent.clone());

        if children.is_none() {
            self.request_mailbox_children(parent);
        }

        children
    }

    fn get_mailbox_children(&self, parent: ParentMailboxId) -> Option<Vec<MailboxId>> {
        let store = self.store.lock().unwrap();
        store
            .mailbox
            .get_children(&parent)
            .map(|children| children.to_owned())
    }

    fn request_mailbox_children(&self, parent: ParentMailboxId) {
        let client = self.client.clone();
        let store = self.store.clone();

        self.task_manager.spawn(
            crate::backend::task_manager::TaskId::QueryChildMailboxes(parent.clone()),
            async move {
                let mut response = {
                    let mut request = client.build();

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

                    match request.send().await {
                        Ok(r) => r,
                        Err(err) => {
                            error!("Couldn't send request to get mailbox children:\n{err}");
                            return;
                        }
                    }
                };

                let mut store = store.lock().unwrap();
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

                store
                    .mailbox
                    .set_children_query_state(parent.clone(), mailbox_get_response.take_state());
                store.mailbox.add_children(parent.clone(), &child_mailboxes);
            },
        );
    }
}
