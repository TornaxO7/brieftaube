use crate::{
    datasource::{
        ThreadRemote,
        jmap::Jmap,
        types::{GetState, remote},
    },
    types::{MailDataCore, MailId, ThreadId},
};

impl ThreadRemote for Jmap {
    async fn fetch_thread(
        &self,
        id: &ThreadId,
    ) -> Result<remote::GetOneResult<remote::GetOneResult<Vec<(MailId, MailDataCore)>>>, Self::Error>
    {
        let mut response = {
            let mut request = self.client.build();

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
                .map(|mut mail| {
                    let id = mail.take_id().into();
                    let data = MailDataCore::from_get_request(mail);
                    (id, data)
                })
                .collect(),
            state: get_email_response.take_state().into(),
        };

        let get_thread_result = remote::GetOneResult {
            value: get_mail_result,
            state: get_thread_response.take_state().into(),
        };

        Ok(get_thread_result)
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
