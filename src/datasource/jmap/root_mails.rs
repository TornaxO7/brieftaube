use crate::{
    datasource::{
        RootMailsRemote,
        jmap::JmapAccount,
        types::{QueryState, QueryWindow, remote},
    },
    types::{MailDataCore, MailId, MailboxId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl RootMailsRemote for JmapAccount {
    async fn fetch_root_mails(
        &self,
        mailbox: &MailboxId,
        window: &QueryWindow,
        calculate_total: bool,
    ) -> Result<remote::QueryResponse<remote::GetOneResult<Vec<MailDataCore>>>> {
        let mut response = {
            let mut request = self.build_request();

            let query_request = request
                .query_email()
                .filter(jmap_client::email::query::Filter::InMailbox {
                    value: mailbox.as_str().to_string(),
                })
                .sort([jmap_client::email::query::Comparator::received_at().descending()])
                .position(window.start as i32)
                .limit(window.limit)
                .calculate_total(calculate_total);

            query_request.arguments().collapse_threads(true);

            let query_result = query_request.result_reference();

            request
                .get_email()
                .ids_ref(query_result)
                .properties(MailDataCore::GET_REQUEST_PROPERTIES);

            request.send().await?
        };

        let mut get_mails_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();
        let mut query_mails_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_query_email()
            .unwrap();

        let get_email_result = remote::GetOneResult {
            value: get_mails_response
                .take_list()
                .into_iter()
                .map(MailDataCore::from_get_request)
                .collect(),
            state: get_mails_response.take_state().into(),
        };

        Ok(remote::QueryResponse {
            value: get_email_result,
            state: query_mails_response.take_query_state().into(),
            total: query_mails_response.total(),
        })
    }

    async fn fetch_root_mails_changes(
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
