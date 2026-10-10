use crate::{
    datasource::{
        ThreadRemote,
        jmap::JmapAccount,
        types::{GetState, remote},
    },
    types::{CachedMail, MailId, MailProperty, ThreadId},
};
use async_trait::async_trait;
use color_eyre::Result;
use std::collections::HashMap;

#[async_trait]
impl ThreadRemote for JmapAccount {
    async fn fetch_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<remote::GetBatchResult<HashMap<ThreadId, Vec<MailId>>, Vec<ThreadId>>> {
        let mut response = {
            let mut request = self.build_request();

            request.get_thread().ids(ids).properties([
                jmap_client::thread::Property::Id,
                jmap_client::thread::Property::EmailIds,
            ]);

            request.send_get_thread().await?
        };

        let mut threads = HashMap::new();
        let not_found: Vec<ThreadId> = response
            .take_not_found()
            .into_iter()
            .map(ThreadId::from)
            .collect();

        for thread in response.take_list() {
            let thread_id = thread.id().into();
            let thread_mail_ids = thread.email_ids().into_iter().map(MailId::from).collect();

            threads.insert(thread_id, thread_mail_ids);
        }

        Ok(remote::GetBatchResult {
            values: threads,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_threads_with_mails(
        &self,
        ids: &[ThreadId],
        properties: Vec<MailProperty>,
    ) -> Result<
        remote::GetBatchResult<
            remote::GetOneResult<HashMap<ThreadId, Vec<CachedMail>>>,
            Vec<ThreadId>,
        >,
    > {
        let mut response = {
            let mut request = self.build_request();

            let thread_mail_ids_result = request
                .get_thread()
                .ids(ids)
                .properties([
                    jmap_client::thread::Property::Id,
                    jmap_client::thread::Property::EmailIds,
                ])
                .result_reference(jmap_client::thread::Property::EmailIds);

            request
                .get_email()
                .ids_ref(thread_mail_ids_result)
                .properties(properties.into_iter().map(Into::into));

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

        let mails_lookup: HashMap<MailId, CachedMail> = email_get_response
            .take_list()
            .into_iter()
            .map(|mail| (mail.id().unwrap().into(), CachedMail::from(mail)))
            .collect();

        let threads_with_mails = {
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
                value: threads_with_mails,
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
