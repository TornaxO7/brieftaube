use super::JmapAccount;
use crate::{
    datasource::{
        MailRemote,
        types::{GetState, remote},
    },
    types::{CachedMail, MailId, MailProperty},
};
use async_trait::async_trait;
use color_eyre::Result;

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

            if properties.contains(&MailProperty::TextBody) {
                request_properties.push(jmap_client::email::Property::BodyValues);
                email_request.arguments().fetch_text_body_values(true);
            }

            if properties.contains(&MailProperty::HtmlBody) {
                request_properties.push(jmap_client::email::Property::BodyValues);
                email_request.arguments().fetch_html_body_values(true);
            }

            email_request.properties(request_properties);

            request.send_get_email().await?
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

    async fn create_mail(&self) {}
}
