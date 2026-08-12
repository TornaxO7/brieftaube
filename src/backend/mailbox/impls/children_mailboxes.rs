use crate::backend::{Backend, MailboxData, ParentMailboxId};

impl Backend {
    pub async fn get_mailbox_children(
        &self,
        parent: ParentMailboxId,
    ) -> Result<Vec<MailboxData>, jmap_client::Error> {
        let children = {
            let store = self.store.lock().unwrap();
            store
                .mailbox
                .get_children_data(&parent)
                .map(|children| children.to_owned())
        };

        match children {
            Some(cached_children) => Ok(cached_children),
            None => {
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
                store.mailbox.add_children(parent.clone(), &child_mailboxes);

                Ok(child_mailboxes)
            }
        }
    }
}
