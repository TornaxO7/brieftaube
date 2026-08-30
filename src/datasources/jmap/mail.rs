use super::Jmap;
use crate::{
    datasources::{
        MailRemote,
        types::{GetState, QueryState, QueryWindow, remote},
    },
    types::{
        MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailNew,
        MailUpdate, MailboxId,
    },
};
use jmap_client::core::set::SetObject;

impl MailRemote for Jmap {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, Vec<MailData>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some(ids))
                .properties(MailData::PROPERTIES);

            request.send_get_email().await?
        };

        let values = response
            .take_list()
            .into_iter()
            .map(MailData::from)
            .collect();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetResult {
            values,
            not_found,
            state: response.take_state(),
        })
    }

    async fn fetch_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<remote::GetResult<MailId, MailDataTextBody>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some([id]))
                .properties(MailDataTextBody::PROPERTIES)
                .arguments()
                .fetch_text_body_values(true);

            request.send_get_email().await?
        };

        let body = response
            .list()
            .into_iter()
            .next()
            .map(MailDataTextBody::new)
            .flatten()
            .unwrap();

        let state = response.take_state();

        Ok(remote::GetResult {
            values: body,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<remote::GetResult<MailId, MailDataHtmlBody>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some([id]))
                .properties(MailDataHtmlBody::PROPERTIES)
                .arguments()
                .fetch_html_body_values(true);

            request.send_get_email().await?
        };

        let body = response
            .list()
            .into_iter()
            .next()
            .map(MailDataHtmlBody::new)
            .flatten()
            .unwrap();

        let state = response.take_state();

        Ok(remote::GetResult {
            values: body,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<remote::GetResult<MailId, Vec<MailDataAttachment>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some([id]))
                .properties([jmap_client::email::Property::Attachments]);

            request.send_get_email().await?
        };

        let values = response
            .take_list()
            .into_iter()
            .next()
            .expect("Mail arrived")
            .attachments()
            .map(|attachments| attachments.iter().map(MailDataAttachment::from).collect())
            .expect("Attachments have been requested");

        let state = response.take_state();

        Ok(remote::GetResult {
            values,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<remote::QueryResponse<MailId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .query_email()
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.0.clone(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()])
                .position(window.start as i32)
                .limit(window.limit)
                .arguments()
                .collapse_threads(true);

            request.send_query_email().await?
        };

        let state = response.take_query_state();
        let ids = response.take_ids().into_iter().map(MailId).collect();

        Ok(remote::QueryResponse { ids, state })
    }

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<remote::SetResult<MailData>, Self::Error> {
        let (mut response, _tmp_id) = {
            let mut request = self.client.build();

            let create = request
                .set_email()
                .if_in_state(since)
                .create()
                .mailbox_ids(new.mailbox_ids);

            if let Some(keywords) = new.keywords {
                create.keywords(keywords);
            }

            for (header, value) in new.headers {
                create.header(header, value);
            }

            let tmp_id = create.create_id().unwrap();
            (request.send_set_email().await?, tmp_id)
        };

        let _state = response.take_new_state();
        todo!("think about this thorough: How to handle errors and what else should be set-able")
    }

    async fn update_mails(
        &self,
        updates: &[MailUpdate],
        since: GetState,
    ) -> Result<remote::UpdateResult, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            let set_mail = request.set_email().if_in_state(since);

            for update in updates {
                let u = set_mail.update(&update.id);

                if let Some(patches) = &update.patch_keywords {
                    for (keyword, set) in patches {
                        u.keyword(keyword.as_str(), *set);
                    }
                }

                if let Some(mailbox_ids) = &update.mailbox_ids {
                    for (id, set) in mailbox_ids {
                        u.mailbox_id(id.as_str(), *set);
                    }
                }
            }

            request.send_set_email().await?
        };

        for update in updates {
            match response.updated(update.id.as_str()) {
                Ok(None) => {}
                Ok(Some(extra)) => {
                    todo!()
                }
                Err(err) => {
                    todo!()
                }
            }
        }

        let new_state = response.take_new_state();

        Ok(remote::UpdateResult { new_state })
    }

    async fn destroy_mails(
        &self,
        ids: Vec<MailId>,
        since: GetState,
    ) -> Result<remote::SetResult<()>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.set_email().if_in_state(since).destroy(ids);
            request.send_set_email().await?
        };

        // TODO: Error handling
        response.unwrap_destroy_errors().unwrap();

        Ok(remote::SetResult {
            value: (),
            state: response.take_new_state(),
        })
    }

    async fn fetch_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.changes_email(since);
            request.send_changes_email().await?
        };

        debug_assert_eq!(
            response.old_state(),
            since.as_str(),
            "TODO: Return custom error"
        );

        let new_state = response.take_new_state();
        let has_more_changes = response.has_more_changes();
        let created = response.take_created().into_iter().map(MailId).collect();
        let updated = response.take_updated().into_iter().map(MailId).collect();
        let destroyed = response.take_destroyed().into_iter().map(MailId).collect();

        Ok(remote::GetChangeResult {
            new_state,
            has_more_changes,
            created,
            updated,
            destroyed,
        })
    }

    async fn fetch_root_mail_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
    ) -> Result<remote::QueryChangeResult<MailId>, Self::Error> {
        let response = {
            let mut request = self.client.build();
            request
                .query_email_changes(since)
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.0.clone(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()]);
            request.send_query_email_changes().await?
        };

        debug_assert_eq!(
            response.old_query_state(),
            since.as_str(),
            "TODO: Refresh query"
        );

        let new_state = response.new_query_state().to_string();
        let removed = response
            .removed()
            .iter()
            .map(|id| MailId(id.clone()))
            .collect();

        let added = response
            .added()
            .into_iter()
            .map(|added| {
                let id = MailId::from(added.id());
                let idx = added.index();

                (id, idx)
            })
            .collect();

        Ok(remote::QueryChangeResult {
            new_state,
            removed,
            added,
        })
    }
}
