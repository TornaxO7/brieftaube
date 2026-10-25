use super::JmapAccount;
use crate::{
    datasource::{
        MailRemote,
        types::{GetState, remote},
    },
    types::{CachedMail, MailId, MailProperty},
};
use async_trait::async_trait;
use color_eyre::{Result, eyre::Context};
use jmap_client::{
    Set,
    core::set::{SetObject, SetRequest},
    email::{Email, EmailBodyPart},
};
use tracing::{error, warn};

#[async_trait]
impl MailRemote for JmapAccount {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
        properties: Vec<MailProperty>,
    ) -> Result<remote::GetBatchResult<Vec<CachedMail>, Vec<MailId>>> {
        let mut response = {
            let mut request = self.build_request();
            let mut request_properties: Vec<jmap_client::email::Property> = properties
                .iter()
                .cloned()
                .map(|property| property.into())
                .collect();

            tracing::debug!("Props: {:?}", properties);

            let email_request = request.get_email().ids(ids);

            let wants_text_body = properties.contains(&MailProperty::TextBody);
            let wants_html_body = properties.contains(&MailProperty::HtmlBody);

            if wants_text_body {
                email_request.arguments().fetch_text_body_values(true);
            }

            if wants_html_body {
                email_request.arguments().fetch_html_body_values(true);
            }

            if wants_html_body || wants_text_body {
                request_properties.push(jmap_client::email::Property::BodyValues);
            }

            email_request.properties(request_properties);

            request
                .send_get_email()
                .await
                .wrap_err("Request Email/get")?
        };

        let values = response
            .take_list()
            .into_iter()
            .map(CachedMail::from)
            .collect();

        tracing::debug!("CachedMails: {:#?}", values);

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetBatchResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn destroy_mails(
        &self,
        ids: &[MailId],
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>> {
        let ids: Vec<MailId> = ids.into_iter().cloned().collect();

        let mut response = {
            let mut request = self.build_request();
            request.set_email().if_in_state(since).destroy(&ids);
            request.send_set_email().await.wrap_err("Send Email/get")?
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
    ) -> Result<remote::GetChangeResult<MailId>> {
        let mut response = {
            let mut request = self.build_request();
            request.changes_email(since.as_ref());
            request
                .send_changes_email()
                .await
                .wrap_err("Request Email/changes")?
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

    async fn create_mail(&self, mail: CachedMail) -> Result<remote::SetResult<CachedMail>> {
        let (mut response, tmp_id) = {
            let mut request = self.build_request();

            let tmp_id = setup_create(request.set_email(), Some(mail.id.clone()), mail.clone());

            (
                request
                    .send_set_email()
                    .await
                    .wrap_err("Request Email/Set (create new draft)")?,
                tmp_id,
            )
        };

        debug_assert_eq!(tmp_id, mail.id.clone());

        if let Err(err) = response.unwrap_create_errors() {
            error!("{}", err);
        }

        Ok(remote::SetResult {
            value: response.created(tmp_id.as_str()).map(CachedMail::from),
            old_state: response
                .old_state()
                .map(|state| GetState::from(state.to_string())),
            new_state: response.take_new_state().into(),
        })
    }

    async fn edit_draft(&self, mail: CachedMail) -> Result<remote::SetResult<CachedMail>> {
        let (mut response, tmp_id) = {
            let mut request = self.build_request();

            let set = request.set_email();

            set.destroy([&mail.id]);
            let tmp_id = setup_create(set, None, mail.clone());

            (
                request
                    .send_set_email()
                    .await
                    .wrap_err("Request Email/Set(destroy + create)")?,
                tmp_id,
            )
        };

        if let Err(err) = response.destroyed(mail.id.as_str()) {
            warn!("Server didn't remove previous mail: {}", err);
        }

        Ok(remote::SetResult {
            value: response.created(tmp_id.as_str()).map(|server_mail| {
                let mut new_mail = mail;
                let server_mail = CachedMail::from(server_mail);
                new_mail.merge(server_mail);
                new_mail
            }),
            old_state: response
                .old_state()
                .map(|state| GetState::from(state.to_string())),
            new_state: response.take_new_state().into(),
        })
    }

    // async fn patch_mails(
    //     &self,
    //     ids: &[MailId],
    //     mailbox_ids: Option<UpdateMailAttribute<MailboxId>>,
    //     keywords: Option<UpdateMailAttribute<MailKeyword>>,
    // ) -> Result<remote::SetResult<()>> {
    //     let mut response = {
    //         let mut request = self.build_request();

    //         let set_email = request.set_email();

    //         for id in ids {
    //             let update = set_email.update(id);

    //             if let Some(mailbox_ids) = mailbox_ids.clone() {
    //                 match mailbox_ids {
    //                     UpdateMailAttribute::Set(ids) => {
    //                         update.mailbox_ids(ids);
    //                     }
    //                     UpdateMailAttribute::Patch(changes) => {
    //                         for (mailbox_id, set) in changes {
    //                             update.mailbox_id(mailbox_id.as_str(), set);
    //                         }
    //                     }
    //                 };
    //             }

    //             if let Some(keywords) = keywords.clone() {
    //                 match keywords {
    //                     UpdateMailAttribute::Set(keywords) => {
    //                         update.keywords(keywords);
    //                     }
    //                     UpdateMailAttribute::Patch(changes) => {
    //                         for (keyword, set) in changes {
    //                             update.keyword(keyword.as_str(), set);
    //                         }
    //                     }
    //                 }
    //             }
    //         }

    //         request.send_set_email().await?
    //     };

    //     response.unwrap_update_errors().unwrap();

    //     Ok(remote::SetResult {
    //         value: Ok(()),
    //         old_state: response.old_state().map(|state| state.to_string().into()),
    //         new_state: response.take_new_state().into(),
    //     })
    // }
}

fn setup_create(
    set_mail: &mut SetRequest<Email<Set>>,
    creation_id: Option<MailId>,
    mail: CachedMail,
) -> MailId {
    let (create, tmp_id) = match creation_id {
        Some(id) => {
            let create = set_mail.create_with_id(&id);
            (create, id)
        }
        None => {
            let create = set_mail.create();
            let id = create.create_id().unwrap().into();
            (create, id)
        }
    };

    create.mailbox_ids(
        mail.mailbox_ids
            .get()
            .expect("Mail is assigned to at least one mailbox"),
    );

    if let Some(keywords) = mail.keywords.get().cloned() {
        create.keywords(keywords);
    }

    if let Some(subject) = mail.subject.get() {
        create.subject(subject);
    }

    if let Some(from) = mail.from.get().cloned() {
        create.from(from);
    }

    if let Some(to) = mail.to.get().cloned() {
        create.to(to);
    }

    if let Some(cc) = mail.cc.get().cloned() {
        create.to(cc);
    }

    if let Some(bcc) = mail.bcc.get().cloned() {
        create.to(bcc);
    }

    if let Some(body_parts) = mail.body_parts.get().cloned() {
        for (id, body_part) in body_parts {
            create.body_value(id, body_part);
        }
    }

    if let Some(text_part_ids) = mail.text_part_ids.get().cloned() {
        for text_part_id in text_part_ids {
            let part = EmailBodyPart::new()
                .part_id(text_part_id)
                .content_type("text/plain");
            create.text_body(part);
        }
    }

    if let Some(html_part_ids) = mail.html_part_ids.get().cloned() {
        for html_part_id in html_part_ids {
            let part = EmailBodyPart::new()
                .part_id(html_part_id)
                .content_type("text/html");
            create.html_body(part);
        }
    }

    tmp_id
}
