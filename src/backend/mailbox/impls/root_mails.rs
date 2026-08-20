use crate::backend::{
    Backend, FetchRole, MailData, MailId, MailboxId, ThreadId,
    mailbox::{self, store::RootMails},
    mails, threads,
    types::RemoteData,
};
use jmap_client::core::{
    query::QueryResponse,
    response::{EmailGetResponse, ThreadGetResponse},
};
use tokio::sync::watch;

impl Backend {
    pub async fn get_mailbox_root_mails(
        &self,
        id: &MailboxId,
    ) -> Result<Vec<MailId>, jmap_client::Error> {
        let role = {
            let mut store = self.store.lock().unwrap();
            let root_mails = store.mailbox.get_root_mails_mut(id);

            match root_mails {
                RemoteData::NotRequested => {
                    let (tx, rx) = watch::channel(());
                    *root_mails = RemoteData::Requested { notifier: rx };
                    FetchRole::Request(tx)
                }
                RemoteData::Requested { notifier } => FetchRole::Wait(notifier.clone()),
                RemoteData::Loaded(root_mails) => {
                    return Ok(root_mails.ids.clone());
                }
            }
        };

        match role {
            FetchRole::Wait(mut notifier) => {
                notifier.changed().await.unwrap();
                let mut store = self.store.lock().unwrap();
                let root_mails = store.mailbox.get_root_mails(id).loaded().unwrap();
                Ok(root_mails.ids.clone())
            }
            FetchRole::Request(notifier) => {
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

                let root_mail_ids = {
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

                    let _ = notifier.send(());
                    store
                        .mailbox
                        .get_root_mails(id)
                        .loaded()
                        .unwrap()
                        .ids
                        .clone()
                };

                Ok(root_mail_ids)
            }
        }
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

        store.add(&id, mail_ids);
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
