use crate::{
    datasource::{
        ThreadRemote,
        jmap::Jmap,
        types::{GetState, remote},
    },
    types::{MailId, ThreadId},
};

impl ThreadRemote for Jmap {
    async fn fetch_threads(
        &self,
        ids: &[ThreadId],
    ) -> Result<remote::GetBatchResult<ThreadId, Vec<(ThreadId, Vec<MailId>)>>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
            request.get_thread().ids(Some(ids));
            request.send_get_thread().await?
        };

        let values = response
            .take_list()
            .into_iter()
            .map(|thread| {
                let id = thread.id().into();
                let thread_mails = thread.email_ids().into_iter().map(MailId::from).collect();
                (id, thread_mails)
            })
            .collect();

        let not_found = response
            .take_not_found()
            .into_iter()
            .map(ThreadId::from)
            .collect();

        Ok(remote::GetBatchResult {
            values,
            not_found,
            state: response.take_state().into(),
        })
    }

    async fn fetch_thread_changes(
        &self,
        since: &GetState,
    ) -> Result<remote::GetChangeResult<ThreadId>, Self::Error> {
        let mut response = {
            let mut request = self.client.build();
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
