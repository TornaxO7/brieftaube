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

    // async fn create_mailbox(&self, new: MailboxNew) -> Result<remote::CreateResult<CachedMailbox>> {
    //     let (mut response, tmp_id) = {
    //         let mut request = self.build_request();
    //         let tmp_id = request
    //             .set_mailbox()
    //             .create()
    //             .name(new.name.as_str())
    //             .parent_id(new.parent_id.clone())
    //             .sort_order(new.sort_order)
    //             .create_id()
    //             .unwrap();

    //         (request.send_set_mailbox().await?, tmp_id)
    //     };

    //     let value = match response.created(&tmp_id) {
    //         Ok(server_mailbox) => Ok(CachedMailbox::from_new(new, server_mailbox)),
    //         Err(err) => {
    //             let jmap_client::Error::Set(error) = err else {
    //                 unreachable!("Weird");
    //             };

    //             Err(error)
    //         }
    //     };

    //     Ok(remote::CreateResult {
    //         value,
    //         state: response.take_new_state().into(),
    //     })
    // }

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
}
