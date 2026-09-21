use crate::{
    datasource::{
        MailboxThreadsRemote,
        jmap::JmapAccount,
        types::{QueryState, QueryWindow, remote},
    },
    types::{MailDataCore, MailId, MailboxId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl MailboxThreadsRemote for JmapAccount {
    async fn fetch_mailbox_threads(
        &self,
        mailbox: &MailboxId,
        window: &QueryWindow,
    ) -> Result<remote::QueryResponse<remote::GetOneResult<Vec<(MailId, MailDataCore)>>>> {
        let mut response = {
            let mut request = self.build_request();

            let query_result_reference = {
                let query_request = request
                    .query_email()
                    .filter(jmap_client::email::query::Filter::InMailbox {
                        value: mailbox.as_str().to_string(),
                    })
                    .sort([jmap_client::email::query::Comparator::received_at().descending()])
                    .position(window.start as i32)
                    .limit(window.limit);
                query_request.arguments().collapse_threads(true);
                query_request.result_reference()
            };

            let thread_ids_result_reference = request
                .get_email()
                .ids_ref(query_result_reference)
                .properties([jmap_client::email::Property::ThreadId])
                .result_reference(jmap_client::email::Property::ThreadId);

            let thread_mail_ids_result_reference = request
                .get_thread()
                .ids_ref(thread_ids_result_reference)
                .result_reference(jmap_client::thread::Property::EmailIds);

            let _thread_mails = request
                .get_email()
                .ids_ref(thread_mail_ids_result_reference)
                .properties(MailDataCore::GET_REQUEST_PROPERTIES);

            request.send().await?
        };

        todo!();
    }

    async fn fetch_mailbox_threads_changes(
        &self,
        mailbox: &MailboxId,
        since: &QueryState,
        up_to_id: Option<&MailId>,
    ) -> Result<remote::QueryChangeResult<MailId>> {
        let response = {
            let mut request = self.build_request();
            let changes = request.query_email_changes(since.as_ref());

            changes
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.0.clone(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()]);

            if let Some(id) = up_to_id {
                changes.up_to_id(id);
            }

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
