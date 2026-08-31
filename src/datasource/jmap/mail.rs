use super::Jmap;
use crate::{
    datasource::{
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
            .map(MailData::from_get_request)
            .collect();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, Vec<(MailId, MailDataTextBody)>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some(ids))
                .properties([
                    jmap_client::email::Property::Id,
                    jmap_client::email::Property::TextBody,
                ])
                .arguments()
                .fetch_text_body_values(true);

            request.send_get_email().await?
        };

        let body = response
            .list()
            .into_iter()
            .map(|server_mail| {
                let id: MailId = server_mail.id().unwrap().into();
                let body = MailDataTextBody::new(server_mail).unwrap();

                (id, body)
            })
            .collect();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetResult {
            values: body,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, Vec<(MailId, MailDataHtmlBody)>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .get_email()
                .ids(Some(ids))
                .properties([
                    jmap_client::email::Property::Id,
                    jmap_client::email::Property::HtmlBody,
                ])
                .arguments()
                .fetch_html_body_values(true);

            request.send_get_email().await?
        };

        let values = response
            .list()
            .into_iter()
            .map(|server_mail| {
                let id: MailId = server_mail.id().unwrap().into();
                let body = MailDataHtmlBody::new(server_mail).unwrap();

                (id, body)
            })
            .collect();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_mails_attachments(
        &self,
        ids: &[MailId],
    ) -> Result<remote::GetResult<MailId, Vec<(MailId, Vec<MailDataAttachment>)>>, Self::Error>
    {
        let mut response = {
            let mut request = self.client.build();

            request.get_email().ids(Some(ids)).properties([
                jmap_client::email::Property::Id,
                jmap_client::email::Property::Attachments,
            ]);

            request.send_get_email().await?
        };

        let values = response
            .take_list()
            .into_iter()
            .map(|server_mail| {
                let id: MailId = server_mail.id().expect("Requested").into();
                let attachments = server_mail
                    .attachments()
                    .expect("Requested")
                    .iter()
                    .map(MailDataAttachment::from)
                    .collect();

                (id, attachments)
            })
            .collect();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: &QueryWindow,
    ) -> Result<remote::QueryResponse<MailId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();

            request
                .query_email()
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.as_str().to_string(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()])
                .position(window.start as i32)
                .limit(window.limit)
                .arguments()
                .collapse_threads(true);

            request.send_query_email().await?
        };
        let ids = response.take_ids().into_iter().map(MailId).collect();

        Ok(remote::QueryResponse {
            ids,
            state: response.take_query_state().into(),
        })
    }

    async fn create_mail(
        &self,
        new: MailNew,
        since: GetState,
    ) -> Result<remote::CreateResult<MailData>, Self::Error> {
        let new2 = new.clone();
        let (mut response, tmp_id) = {
            let mut request = self.client.build();

            let create = request
                .set_email()
                .if_in_state(since)
                .create()
                .mailbox_ids(new.mailbox_ids);

            create.keywords(new.keywords);

            if let Some(from) = new.from {
                create.from(from.0);
            }

            if let Some(to) = new.to {
                create.to(to.0);
            }

            if let Some(cc) = new.cc {
                create.cc(cc.0);
            }

            if let Some(bcc) = new.bcc {
                create.bcc(bcc.0);
            }

            if let Some(subject) = new.subject {
                create.subject(subject);
            }

            if let Some(in_reply_to) = new.in_reply_to {
                create.in_reply_to(in_reply_to);
            }

            if let Some(references) = new.references {
                create.references(references);
            }

            let tmp_id = create.create_id().unwrap();
            (request.send_set_email().await?, tmp_id)
        };

        let value = match response.created(tmp_id.as_str()) {
            Ok(server_mail) => Ok(MailData::from_new(new2, server_mail)),
            Err(err) => {
                let jmap_client::Error::Set(error) = err else {
                    unreachable!("Why... are we getting another error???");
                };
                Err(error)
            }
        };

        Ok(remote::CreateResult {
            value,
            state: response.take_new_state().into(),
        })
    }

    async fn update_mails(
        &self,
        updates: Vec<(MailData, MailUpdate)>,
        since: GetState,
    ) -> Result<remote::UpdateResult<MailId, MailData>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            let set_mail = request.set_email().if_in_state(since);

            for (data, update) in updates.iter() {
                let u = set_mail.update(&data.id);

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
                        "Server responded with additional changes:{:#?}\nNot implemented yet :/\nPlease create an issue!",
                        extra
                    );

                    data.update(update);
                    updated.push(data);
                }
                Err(err) => {
                    let jmap_client::Error::Set(error) = err else {
                        unreachable!("Why... are we getting another error???");
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

    async fn destroy_mails(
        &self,
        ids: Vec<MailId>,
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.set_email().if_in_state(since).destroy(&ids);
            request.send_set_email().await?
        };

        let mut destroyed = Vec::new();
        let mut failed = Vec::new();

        for id in ids {
            match response.destroyed(id.as_str()) {
                Ok(()) => destroyed.push(id),
                Err(err) => {
                    let jmap_client::Error::Set(error) = err else {
                        unreachable!("Unknown error return for destroying");
                    };
                    failed.push((id, error));
                }
            }
        }

        Ok(remote::DestroyResult {
            destroyed,
            failed,
            new_state: response.take_new_state().into(),
        })
    }

    async fn fetch_mail_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<MailId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.changes_email(since.as_ref());
            request.send_changes_email().await?
        };

        debug_assert_eq!(
            response.old_state(),
            since.0.as_str(),
            "TODO: Return custom error"
        );

        let has_more_changes = response.has_more_changes();
        let created = response.take_created().into_iter().map(MailId).collect();
        let updated = response.take_updated().into_iter().map(MailId).collect();
        let destroyed = response.take_destroyed().into_iter().map(MailId).collect();

        Ok(remote::GetChangeResult {
            new_state: response.take_new_state().into(),
            has_more_changes,
            created,
            updated,
            destroyed,
        })
    }

    async fn fetch_root_mails_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
    ) -> Result<remote::QueryChangeResult<MailId>, Self::Error> {
        todo!("Add `uptold` option");

        let response = {
            let mut request = self.client.build();
            request
                .query_email_changes(since.as_ref())
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.0.clone(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()]);
            request.send_query_email_changes().await?
        };

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
            new_state: response.new_query_state().to_string().into(),
            removed,
            added,
        })
    }
}
