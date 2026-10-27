use crate::{
    datasource::{
        MailboxRemote,
        jmap::JmapAccount,
        types::{GetState, remote},
    },
    types::{CachedMailbox, MailboxId, MailboxProperty},
};
use async_trait::async_trait;
use color_eyre::Result;
use jmap_client::core::request::ResultReference;
use tracing::instrument;

#[async_trait]
impl MailboxRemote for JmapAccount {
    #[instrument(skip(self))]
    async fn fetch_mailboxes_all(
        &self,
        properties: Vec<MailboxProperty>,
    ) -> Result<remote::GetOneResult<Vec<CachedMailbox>>> {
        let mut response = {
            let mut request = self.build_request();
            request
                .get_mailbox()
                .properties(properties.into_iter().map(|property| property.into()));
            request.send_get_mailbox().await?
        };

        Ok(remote::GetOneResult {
            value: response
                .take_list()
                .into_iter()
                .map(CachedMailbox::from)
                .collect(),
            state: response.take_state().into(),
        })
    }

    async fn fetch_mailboxes(
        &self,
        ids: &[MailboxId],
        properties: Vec<MailboxProperty>,
    ) -> Result<remote::GetBatchResult<Vec<CachedMailbox>, Vec<MailboxId>>> {
        let mut response = {
            let mut request = self.build_request();

            request
                .get_mailbox()
                .ids(ids)
                .properties(properties.into_iter().map(|property| property.into()));

            request.send_get_mailbox().await?
        };

        Ok(remote::GetBatchResult {
            values: response
                .take_list()
                .into_iter()
                .map(CachedMailbox::from)
                .collect(),
            not_found: response
                .take_not_found()
                .into_iter()
                .map(MailboxId::from)
                .collect(),
            state: response.take_state().into(),
        })
    }

    async fn fetch_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailboxId>> {
        let mut response = {
            let mut request = self.build_request();
            request.changes_mailbox(since.as_ref());
            request.send_changes_mailbox().await?
        };

        debug_assert_eq!(response.old_state(), since.as_ref());

        Ok(remote::GetChangeResult {
            new_state: response.take_new_state().into(),
            has_more_changes: response.has_more_changes(),
            created: response.take_created().into_iter().map(MailboxId).collect(),
            updated: response.take_updated().into_iter().map(MailboxId).collect(),
            destroyed: response
                .take_destroyed()
                .into_iter()
                .map(MailboxId)
                .collect(),
        })
    }

    async fn destroy_mailboxes(
        &self,
        ids: &[MailboxId],
        on_destroy_remove_emails: bool,
    ) -> Result<remote::DestroyResult<MailboxId>> {
        let mut response = {
            let mut request = self.build_request();
            request
                .set_mailbox()
                .destroy(ids)
                .arguments()
                .on_destroy_remove_emails(on_destroy_remove_emails);
            request.send_set_mailbox().await?
        };

        let mut destroyed = Vec::new();
        let mut failed = Vec::new();

        for id in ids {
            match response.destroyed(id.as_str()) {
                Ok(()) => destroyed.push(id.clone()),
                Err(err) => {
                    let jmap_client::Error::Set(error) = err else {
                        unreachable!();
                    };
                    failed.push((id.clone(), error));
                }
            }
        }

        Ok(remote::DestroyResult {
            destroyed,
            failed,
            new_state: response.take_new_state().into(),
        })
    }

    async fn create_mailbox(
        &self,
        mut new_mailbox: CachedMailbox,
    ) -> Result<remote::SetResult<CachedMailbox>> {
        let mut response = {
            let mut request = self.build_request();

            let set = request.set_mailbox().create_with_id(new_mailbox.id.clone());

            if let Some(name) = new_mailbox.name.get().cloned() {
                set.name(name);
            }

            if let Some(role) = new_mailbox.role.get().cloned() {
                set.role(role);
            }

            if let Some(sort_order) = new_mailbox.sort_order.get().cloned() {
                set.sort_order(sort_order);
            }

            if let Some(parent_id) = new_mailbox.parent_id.get().cloned() {
                set.parent_id(parent_id);
            }

            request
                .get_mailbox()
                // fine, I'm gonna do the result reference myself >:(
                .ids_ref(ResultReference::new(
                    jmap_client::Method::SetMailbox,
                    0,
                    format!("/created/{}/id", new_mailbox.id.as_str()),
                ))
                .properties(
                    CachedMailbox::PROPERTIES
                        .into_iter()
                        .map(|property| property.into()),
                );

            request.send().await?
        };

        let mut get_method_call = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_mailbox()
            .unwrap();

        let mut set_method_call = response
            .pop_method_response()
            .unwrap()
            .unwrap_set_mailbox()
            .unwrap();

        let created_mailbox = get_method_call
            .take_list()
            .into_iter()
            .next()
            .map(CachedMailbox::from)
            .unwrap();

        new_mailbox.merge(created_mailbox);

        Ok(remote::SetResult {
            value: Ok(new_mailbox),
            old_state: set_method_call.old_state().map(GetState::from),
            new_state: set_method_call.take_new_state().into(),
        })
    }
}
