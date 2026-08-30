use jmap_client::core::set::SetObject;

use super::Jmap;
use crate::{
    datasources::{
        MailRemote,
        types::{
            GetChangeResult, GetState, QueryChangeResult, QueryState, QueryWindow, RemoteGetResult,
            RemoteQueryResponse, RemoteSetResult,
        },
    },
    types::{
        MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailNew,
        MailUpdate, MailboxId,
    },
};

impl MailRemote for Jmap {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
    ) -> Result<RemoteGetResult<MailId, Vec<MailData>>, Self::Error> {
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

        Ok(RemoteGetResult {
            values,
            not_found,
            state: response.take_state(),
        })
    }

    async fn fetch_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, MailDataTextBody>, Self::Error> {
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

        Ok(RemoteGetResult {
            values: body,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, MailDataHtmlBody>, Self::Error> {
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

        Ok(RemoteGetResult {
            values: body,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<RemoteGetResult<MailId, Vec<MailDataAttachment>>, Self::Error> {
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

        Ok(RemoteGetResult {
            values,
            not_found: vec![],
            state,
        })
    }

    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<RemoteQueryResponse<MailId>, Self::Error> {
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

        Ok(RemoteQueryResponse { ids, state })
    }

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<RemoteSetResult<MailData>, Self::Error> {
        let (mut response, _tmp_id) = {
            let mut request = self.client.build();

            let create = request
                .set_email()
                .if_in_state(since)
                .create()
                .mailbox_ids(new.mailbox_ids);

            // TODO: extend the options
            if let Some(keywords) = new.keywords {
                create.keywords(keywords);
            }

            let tmp_id = create.create_id().unwrap();
            (request.send_set_email().await?, tmp_id)
        };

        let _state = response.take_new_state();
        todo!("think about options")
    }

    async fn update_mail(
        &self,
        update: MailUpdate,
        since: GetState,
    ) -> Result<RemoteSetResult<()>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            let u = request.set_email().if_in_state(since).update(update.id);

            if let Some(patches) = update.patch_keywords {
                for (keyword, set) in patches {
                    u.keyword(keyword.as_str(), set);
                }
            }

            if let Some(mailbox_ids) = update.mailbox_ids {
                for (id, set) in mailbox_ids {
                    u.mailbox_id(id.as_str(), set);
                }
            }

            request.send_set_email().await?
        };

        response.unwrap_update_errors()?;

        Ok(RemoteSetResult {
            value: (),
            state: response.take_new_state(),
        })
    }

    async fn destroy_mail(
        &self,
        id: &MailId,
        since: GetState,
    ) -> Result<RemoteSetResult<()>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.set_email().if_in_state(since).destroy([id]);
            request.send_set_email().await?
        };

        todo!()
    }

    async fn fetch_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<GetChangeResult<MailId>, Self::Error> {
        todo!()
    }

    async fn fetch_root_mail_changes(
        &self,
        since: &QueryState,
    ) -> Result<QueryChangeResult<MailId>, Self::Error> {
        todo!()
    }
}
