use crate::{
    datasource::{
        MailboxRemote,
        jmap::Jmap,
        types::{GetState, remote},
    },
    types::{MailboxData, MailboxId, MailboxNew, MailboxUpdate},
};
use jmap_client::core::set::SetObject;

impl MailboxRemote for Jmap {
    async fn fetch_mailbox_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailboxId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
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

    async fn create_mailbox(
        &self,
        new: MailboxNew,
    ) -> Result<remote::CreateResult<MailboxData>, Self::Error> {
        let (mut response, tmp_id) = {
            let mut request = self.client.build();
            let tmp_id = request
                .set_mailbox()
                .create()
                .name(new.name.as_str())
                .parent_id(new.parent_id.clone())
                .sort_order(new.sort_order)
                .create_id()
                .unwrap();

            (request.send_set_mailbox().await?, tmp_id)
        };

        let value = match response.created(&tmp_id) {
            Ok(server_mailbox) => Ok(MailboxData::from_new(new, server_mailbox)),
            Err(err) => {
                let jmap_client::Error::Set(error) = err else {
                    unreachable!("Weird");
                };

                Err(error)
            }
        };

        Ok(remote::CreateResult {
            value,
            state: response.take_new_state().into(),
        })
    }

    async fn update_mailboxes(
        &self,
        updates: Vec<(MailboxData, MailboxUpdate)>,
        since: &GetState,
    ) -> Result<remote::UpdateResult<MailboxId, MailboxData>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            let set = request.set_mailbox().if_in_state(since.as_ref());

            for (data, update) in &updates {
                let u = set.update(&data.id);

                if let Some(name) = &update.name {
                    u.name(name);
                }

                if let Some(role) = update.role.clone() {
                    u.role(role);
                }

                if let Some(sort_order) = update.sort_order {
                    u.sort_order(sort_order);
                }

                if let Some(parent_id) = update.parent_id.clone() {
                    u.parent_id(parent_id);
                }
            }

            request.send_set_mailbox().await?
        };

        let mut updated = Vec::new();
        let mut failed = Vec::new();

        for (mut data, update) in updates {
            let id = data.id.clone();
            match response.updated(id.as_str()) {
                Ok(None) => {
                    data.update(update);
                    updated.push(data);
                }
                Ok(Some(extra)) => {
                    tracing::warn!(
                        "The server responded with extra data to be updated:{:#?}\nMight not take everything :/",
                        extra
                    );

                    data.update(update);

                    if let Some(my_rights) = extra.my_rights() {
                        data.my_rights = my_rights.clone();
                    }

                    updated.push(data);
                }
                Err(err) => {
                    let jmap_client::Error::Set(error) = err else {
                        unreachable!("No");
                    };

                    failed.push((id, error));
                }
            }
        }

        Ok(remote::UpdateResult {
            updated,
            failed,
            new_state: response.take_new_state().into(),
        })
    }

    async fn destroy_mailboxes(
        &self,
        ids: &[MailboxId],
        on_destroy_remove_emails: bool,
    ) -> Result<remote::DestroyResult<MailboxId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
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
