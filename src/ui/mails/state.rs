use crate::{backend, ui::MailboxId};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

pub struct State {
    account: Arc<backend::Account>,

    rx: mpsc::Receiver<Vec<Email>>,
    tx: Arc<mpsc::Sender<Vec<Email>>>,

    /// `None`: Means that it's currently requested but the response didn't arrive yet.
    mails: HashMap<MailboxId, Vec<Email>>,

    selected_mailbox_id: Option<MailboxId>,
    list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        Self {
            account,

            selected_mailbox_id: None,

            rx,
            tx: Arc::new(tx),

            mails: HashMap::new(),

            list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn open_mailbox(&mut self, mailbox_id: MailboxId) {
        self.selected_mailbox_id = Some(mailbox_id.clone());

        let account = self.account.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let client = &account.client;

            let initial_mails_ids = {
                let mut request = client.build();
                let query = request.query_email();
                query.arguments().collapse_threads(true);
                query
                    .filter(jmap_client::email::query::Filter::in_mailbox(mailbox_id))
                    .limit(10)
                    .calculate_total(true)
                    .position(0)
                    .sort([jmap_client::email::query::Comparator::received_at().descending()]);

                request.send_query_email().await.unwrap()
            };

            let mails_with_thread_id = {
                let mut request = client.build();
                request
                    .get_email()
                    .ids(Some(initial_mails_ids.ids()))
                    .properties([jmap_client::email::Property::ThreadId]);

                request.send_get_email().await.unwrap()
            };

            let thread_mail_ids = {
                let thread_ids = mails_with_thread_id
                    .list()
                    .iter()
                    .map(|mail| mail.thread_id().unwrap())
                    .collect::<Vec<&str>>();

                let mut request = client.build();
                request.get_thread().ids(Some(thread_ids));

                request.send_get_thread().await.unwrap()
            };

            let thread_mails = {
                let mut mails = Vec::new();

                for thread_mail_id in thread_mail_ids.list() {
                    let mut request = client.build();
                    request
                        .get_email()
                        .ids(Some(thread_mail_id.email_ids()))
                        .properties([
                            jmap_client::email::Property::Subject,
                            jmap_client::email::Property::From,
                            jmap_client::email::Property::ReceivedAt,
                            jmap_client::email::Property::Preview,
                            jmap_client::email::Property::ThreadId,
                        ]);

                    let mut response = request.send_get_email().await.unwrap();
                    mails.extend(response.take_list());
                }

                mails
            };

            tx.send(thread_mails).await.unwrap();
        });
    }

    pub fn update(&mut self) {
        if let Some(selected_mailbox_id) = self.selected_mailbox_id.clone() {
            match self.rx.try_recv() {
                Ok(mails) => {
                    self.mails.insert(selected_mailbox_id.to_string(), mails);
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
            }
        }
    }

    pub fn get_render_mail_list_data(
        &mut self,
    ) -> Option<(&Vec<Email>, &mut tui_widget_list::ListState)> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                return Some((mails, &mut self.list_state));
            }
        }

        None
    }

    pub fn get_render_preview_data(&self) -> Option<&str> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                if let Some(idx) = self.list_state.selected {
                    return Some(mails[idx].preview().unwrap());
                }
            }
        }

        None
    }

    pub fn select_next_mail(&mut self) {
        self.list_state.next();
    }

    pub fn select_previous_mail(&mut self) {
        self.list_state.previous();
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
