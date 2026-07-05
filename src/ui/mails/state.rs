use crate::{backend, ui::MailboxId};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

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
        self.list_state.selected = None;

        let account = self.account.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let client = &account.client;

            let initial_mails_ids = {
                let mut request = client.build();
                let query = request.query_email();
                query
                    .filter(jmap_client::email::query::Filter::in_mailbox(mailbox_id))
                    .limit(100)
                    .calculate_total(true)
                    .position(0)
                    .sort([jmap_client::email::query::Comparator::received_at().descending()]);

                request.send_query_email().await.unwrap()
            };

            let mut mails = {
                let mut request = client.build();
                request
                    .get_email()
                    .ids(Some(initial_mails_ids.ids()))
                    .properties([
                        jmap_client::email::Property::Subject,
                        jmap_client::email::Property::From,
                        jmap_client::email::Property::ReceivedAt,
                        jmap_client::email::Property::Preview,
                        jmap_client::email::Property::ThreadId,
                    ]);

                request.send_get_email().await.unwrap()
            };

            tx.send(mails.take_list()).await.unwrap();
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

        if let Some(selected_mailbox_id) = self.selected_mailbox_id.as_ref() {
            let select_first_entry =
                self.mails.get(selected_mailbox_id).is_some() && self.list_state.selected.is_none();

            if select_first_entry {
                self.list_state.next();
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
                    return mails.get(idx).map(|mail| mail.preview().unwrap());
                }
            }
        }

        None
    }

    pub fn get_selected_mail(&self) -> Option<&Email> {
        if let Some(id) = self.selected_mailbox_id.as_ref() {
            if let Some(mails) = self.mails.get(id) {
                if let Some(idx) = self.list_state.selected {
                    return mails.get(idx);
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
