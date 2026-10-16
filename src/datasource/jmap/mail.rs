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
use jmap_client::{
    Set,
    core::set::{SetObject, SetRequest},
    email::Email,
};
use tracing::warn;

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

    async fn create_mail(&self, mail: CachedMail) -> Result<remote::SetResult<CachedMail>> {
        let (mut response, tmp_id) = {
            let mut request = self.build_request();

            let tmp_id = setup_create(request.set_email(), Some(mail.id.clone()), mail.clone());

            (request.send_set_email().await?, tmp_id)
        };

        debug_assert_eq!(tmp_id, mail.id.clone());

        if let Err(err) = response.unwrap_create_errors() {
            tracing::error!("{}", err);
        }

        Ok(remote::SetResult {
            value: response.created(tmp_id.as_str()).map(CachedMail::from),
            old_state: response
                .old_state()
                .map(|state| GetState::from(state.to_string())),
            new_state: response.take_new_state().into(),
        })
    }

    async fn edit_mail(&self, mail: CachedMail) -> Result<remote::SetResult<CachedMail>> {
        let (mut response, tmp_id) = {
            let mut request = self.build_request();

            let set = request.set_email();

            set.destroy([&mail.id]);
            let tmp_id = setup_create(set, None, mail.clone());

            (request.send_set_email().await?, tmp_id)
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

    tmp_id
}
