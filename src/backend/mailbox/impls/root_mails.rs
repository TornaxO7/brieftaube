use crate::backend::{
    Backend, MailData, MailId, MailboxId, ThreadId, mailbox, mailbox::store::RootMails, mails,
    threads, types::CollapsedMail,
};
use jmap_client::core::{
    query::QueryResponse,
    response::{EmailGetResponse, ThreadGetResponse},
};

impl Backend {
    pub async fn get_or_request_mailbox_root_mails(
        &self,
        id: &MailboxId,
    ) -> Result<Vec<CollapsedMail>, jmap_client::Error> {
        match self.get_mailbox_root_mails(id) {
            Some(cached_root_mails) => Ok(cached_root_mails),
            None => {
                let mut response = {
                    let mut request = self.client.build();

                    let mail_query_result = {
                        let query_mail = request
                            .query_email()
                            .filter(jmap_client::email::query::Filter::InMailbox {
                                value: id.clone().0,
                            })
                            .sort([
                                jmap_client::email::query::Comparator::received_at().descending()
                            ])
                            .position(0)
                            .limit(10);
                        query_mail.arguments().collapse_threads(true);
                        query_mail.result_reference()
                    };

                    let thread_ids = request
                        .get_email()
                        .ids_ref(mail_query_result)
                        .properties(MailData::PROPERTIES)
                        .result_reference(jmap_client::email::Property::ThreadId);

                    request.get_thread().ids_ref(thread_ids);

                    request.send().await?
                };

                {
                    let mut store = self.store.lock().unwrap();

                    handle_thread_response(
                        &mut store.threads,
                        response
                            .pop_method_response()
                            .unwrap()
                            .unwrap_get_thread()
                            .unwrap(),
                    );

                    handle_mail_response(
                        &mut store.mails,
                        response
                            .pop_method_response()
                            .unwrap()
                            .unwrap_get_email()
                            .unwrap(),
                    );

                    handle_root_mails_response(
                        &mut store.mailbox,
                        &id,
                        response
                            .pop_method_response()
                            .unwrap()
                            .unwrap_query_email()
                            .unwrap(),
                    );
                }

                Ok(self.get_mailbox_root_mails(id).unwrap())
            }
        }
    }

    fn get_mailbox_root_mails(&self, id: &MailboxId) -> Option<Vec<CollapsedMail>> {
        let store = self.store.lock().unwrap();
        store.mailbox.get_root_mails(id).map(|root_mails| {
            let mut collapsed_mails: Vec<CollapsedMail> = Vec::with_capacity({
                store
                    .mailbox
                    .get_data(id)
                    .map(|mailbox| mailbox.total_threads)
                    .unwrap_or(16)
            });

            for root_mail_id in &root_mails.ids {
                let root_mail = store.mails.get(&root_mail_id).expect("Requested");
                let root_mail_thread = store
                    .threads
                    .get_mails(&root_mail.thread_id)
                    .expect("Requested");

                let thread_has_only_one_mail = root_mail_thread.len() == 1;
                let entry = if thread_has_only_one_mail {
                    CollapsedMail::SingleMail(root_mail_id.clone())
                } else {
                    CollapsedMail::CollapsedThread(
                        root_mail_id.clone(),
                        root_mail.thread_id.clone(),
                    )
                };

                collapsed_mails.push(entry);
            }

            collapsed_mails
        })
    }
}

fn handle_thread_response(store: &mut threads::Store, mut response: ThreadGetResponse) {
    for thread in response.take_list() {
        let id = ThreadId(thread.id().to_owned());
        let mail_ids = thread
            .email_ids()
            .into_iter()
            .map(|id| MailId(id.clone()))
            .collect();

        store.insert(id, mail_ids);
    }

    store.set_state(response.take_state());
}

fn handle_mail_response(store: &mut mails::Store, mut response: EmailGetResponse) {
    for mail in response.take_list() {
        store.add(MailData::from(mail));
    }

    store.set_state(response.take_state());
}

fn handle_root_mails_response(store: &mut mailbox::Store, id: &MailboxId, response: QueryResponse) {
    store.set_root_mails(id.clone(), RootMails::new(response));
}
