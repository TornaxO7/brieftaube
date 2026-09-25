use std::collections::HashMap;

use crate::{
    datasource::{
        ThreadRemote,
        jmap::JmapAccount,
        types::{GetState, remote},
    },
    types::{MailDataCore, MailId, ThreadId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl ThreadRemote for JmapAccount {
    async fn fetch_thread(
        &self,
        id: &ThreadId,
    ) -> Result<remote::GetOneResult<remote::GetOneResult<Vec<MailDataCore>>>> {
        let mut response = {
            let mut request = self.build_request();

            let thread_mail_ids_ref = request
                .get_thread()
                .ids(Some([id]))
                .result_reference(jmap_client::thread::Property::EmailIds);
            request
                .get_email()
                .ids_ref(thread_mail_ids_ref)
                .properties(MailDataCore::GET_REQUEST_PROPERTIES);

            request.send().await?
        };

        let mut get_email_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();

        let mut get_thread_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_thread()
            .unwrap();

        let get_mail_result = remote::GetOneResult {
            value: get_email_response
                .take_list()
                .into_iter()
                .map(MailDataCore::from_get_request)
                .collect(),
            state: get_email_response.take_state().into(),
        };

        let get_thread_result = remote::GetOneResult {
            value: get_mail_result,
            state: get_thread_response.take_state().into(),
        };

        Ok(get_thread_result)
    }

    async fn fetch_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<
        remote::GetBatchResult<
            remote::GetOneResult<HashMap<ThreadId, Vec<MailDataCore>>>,
            Vec<ThreadId>,
        >,
    > {
        let mut response = {
            let mut request = self.build_request();

            let thread_mail_ids_result = request
                .get_thread()
                .ids(Some(ids))
                .properties([jmap_client::thread::Property::Id])
                .result_reference(jmap_client::thread::Property::EmailIds);

            request
                .get_email()
                .ids_ref(thread_mail_ids_result)
                .properties(MailDataCore::GET_REQUEST_PROPERTIES);

            request.send().await?
        };

        // unwrap response

        let mut email_get_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_email()
            .unwrap();
        let mut thread_get_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_get_thread()
            .unwrap();

        let mails_lookup: HashMap<MailId, MailDataCore> = email_get_response
            .take_list()
            .into_iter()
            .map(MailDataCore::from_get_request)
            .map(|mail| (mail.id.clone(), mail))
            .collect();

        let threads = {
            let mut threads = HashMap::with_capacity(thread_get_response.list().len());

            for remote_thread in thread_get_response.take_list() {
                let thread_mails = remote_thread
                    .email_ids()
                    .iter()
                    .map(|id| MailId::from(id))
                    .map(|id| mails_lookup.get(&id).unwrap().clone())
                    .collect();

                threads.insert(remote_thread.id().into(), thread_mails);
            }

            threads
        };

        Ok(remote::GetBatchResult {
            values: remote::GetOneResult {
                value: threads,
                state: email_get_response.take_state().into(),
            },
            not_found: thread_get_response
                .take_not_found()
                .into_iter()
                .map(ThreadId)
                .collect(),
            state: thread_get_response.take_state().into(),
        })
    }

    async fn fetch_thread_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<ThreadId>> {
        let mut response = {
            let mut request = self.build_request();
            request.changes_thread(since.as_ref());
            request.send_changes_thread().await?
        };

        Ok(remote::GetChangeResult {
            new_state: response.take_new_state().into(),
            has_more_changes: response.has_more_changes(),
            created: response
                .take_created()
                .into_iter()
                .map(ThreadId::from)
                .collect(),
            updated: response
                .take_updated()
                .into_iter()
                .map(ThreadId::from)
                .collect(),
            destroyed: response
                .take_destroyed()
                .into_iter()
                .map(ThreadId::from)
                .collect(),
        })
    }
}
