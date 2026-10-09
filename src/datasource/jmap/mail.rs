use super::JmapAccount;
use crate::{
    datasource::{
        MailRemote,
        types::{GetState, remote},
    },
    types::MailId,
};
use async_trait::async_trait;
use color_eyre::Result;
use jmap_client::email::Property;

#[async_trait]
impl MailRemote for JmapAccount {
    async fn fetch_mails(
        &self,
        ids: &[MailId],
        properties: Vec<Property>,
    ) -> Result<remote::GetBatchResult<Vec<jmap_client::email::Email>, Vec<MailId>>> {
        let mut response = {
            let mut request = self.build_request();

            let email_request = request.get_email().ids(Some(ids));

            if properties.contains(&Property::TextBody) {
                email_request.arguments().fetch_text_body_values(true);
            }

            if properties.contains(&Property::HtmlBody) {
                email_request.arguments().fetch_html_body_values(true);
            }

            email_request.properties(properties);

            request.send_get_email().await?
        };

        let values = response.take_list();

        let not_found = response.take_not_found().into_iter().map(MailId).collect();

        Ok(remote::GetBatchResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    // async fn create_mail(
    //     &self,
    //     new: MailNew,
    //     since: GetState,
    // ) -> Result<remote::CreateResult<MailData>, {
    //     let new2 = new.clone();
    //     let (mut response, tmp_id) = {
    //         let mut request = self.build_request();

    //         let create = request
    //             .set_email()
    //             .if_in_state(since)
    //             .create()
    //             .mailbox_ids(new.mailbox_ids);

    //         create.keywords(new.keywords);

    //         if let Some(from) = new.from {
    //             create.from(from.0);
    //         }

    //         if let Some(to) = new.to {
    //             create.to(to.0);
    //         }

    //         if let Some(cc) = new.cc {
    //             create.cc(cc.0);
    //         }

    //         if let Some(bcc) = new.bcc {
    //             create.bcc(bcc.0);
    //         }

    //         if let Some(subject) = new.subject {
    //             create.subject(subject);
    //         }

    //         if let Some(in_reply_to) = new.in_reply_to {
    //             create.in_reply_to(in_reply_to);
    //         }

    //         if let Some(references) = new.references {
    //             create.references(references);
    //         }

    //         let tmp_id = create.create_id().unwrap();
    //         (request.send_set_email().await?, tmp_id)
    //     };

    //     let value = match response.created(tmp_id.as_str()) {
    //         Ok(server_mail) => Ok(MailData::from_new(new2, server_mail)),
    //         Err(err) => {
    //             let jmap_client::Error::Set(error) = err else {
    //                 unreachable!("Why... are we getting another error???");
    //             };
    //             Err(error)
    //         }
    //     };

    //     Ok(remote::CreateResult {
    //         value,
    //         state: response.take_new_state().into(),
    //     })
    // }

    // async fn update_mails(
    //     &self,
    //     updates: Vec<(MailData, MailUpdate)>,
    //     since: GetState,
    // ) -> Result<remote::UpdateResult<MailId, MailData>, {
    //     let mut response = {
    //         let mut request = self.build_request();
    //         let set_mail = request.set_email().if_in_state(since);

    //         for (data, update) in updates.iter() {
    //             let u = set_mail.update(&data.id);

    //             if let Some(patches) = &update.patch_keywords {
    //                 for (keyword, set) in patches {
    //                     u.keyword(keyword.as_str(), *set);
    //                 }
    //             }

    //             if let Some(mailbox_ids) = &update.mailbox_ids {
    //                 for (id, set) in mailbox_ids {
    //                     u.mailbox_id(id.as_str(), *set);
    //                 }
    //             }
    //         }

    //         request.send_set_email().await?
    //     };

    //     let mut updated = Vec::new();
    //     let mut failed = Vec::new();

    //     for (mut data, update) in updates {
    //         let id = data.id.clone();
    //         match response.updated(id.as_str()) {
    //             Ok(None) => {
    //                 data.update(update);
    //                 updated.push(data);
    //             }
    //             Ok(Some(extra)) => {
    //                 tracing::warn!(
    //                     "Server responded with additional changes:{:#?}\nNot implemented yet :/\nPlease create an issue!",
    //                     extra
    //                 );

    //                 data.update(update);
    //                 updated.push(data);
    //             }
    //             Err(err) => {
    //                 let jmap_client::Error::Set(error) = err else {
    //                     unreachable!("Why... are we getting another error???");
    //                 };

    //                 failed.push((id, error));
    //             }
    //         }
    //     }

    //     Ok(remote::UpdateResult {
    //         updated,
    //         failed,
    //         new_state: response.take_new_state().into(),
    //     })
    // }

    async fn destroy_mails(
        &self,
        ids: &[MailId],
        since: GetState,
    ) -> Result<remote::DestroyResult<MailId>> {
        let ids: Vec<MailId> = ids.into_iter().cloned().collect();

        let mut response = {
            let mut request = self.build_request();
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
    ) -> Result<remote::GetChangeResult<MailId>> {
        let mut response = {
            let mut request = self.build_request();
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
}
