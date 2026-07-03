use crate::{backend, ui::MailboxId};
use jmap_client::email::Email;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

pub struct State {
    account: Arc<backend::Account>,

    rx_mails: mpsc::Receiver<Vec<Email>>,
    tx_mails: Arc<mpsc::Sender<Vec<Email>>>,

    /// `None`: Means that it's currently requested but the response didn't arrive yet.
    mails: HashMap<MailboxId, Option<Vec<Email>>>,

    selected_mailbox_id: Option<MailboxId>,
    list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx_mails, rx_mails) = mpsc::channel(1);

        Self {
            account,

            rx_mails,
            tx_mails: Arc::new(tx_mails),

            mails: HashMap::new(),

            selected_mailbox_id: None,
            list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn open_mailbox(&mut self, mailbox_id: MailboxId) {
        self.selected_mailbox_id = Some(mailbox_id);
    }

    pub fn update(&mut self) {
        if let Some(selected_mailbox_id) = self.selected_mailbox_id.clone() {
            match self.rx_mails.try_recv() {
                Ok(mails) => {
                    self.mails
                        .insert(selected_mailbox_id.to_string(), Some(mails));
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
            }
        }
    }

    pub fn get_mails<'a>(&mut self) -> Option<Vec<Email>> {
        if let Some(selected_mailbox_id) = self.selected_mailbox_id.clone() {
            if let Some(mails) = self.mails.get(&selected_mailbox_id) {
                if let Some(mails) = mails {
                    return Some(mails.clone());
                }
            } else {
                // mark that we start requesting the resource
                self.mails.insert(selected_mailbox_id.clone(), None);

                let account = self.account.clone();
                let tx_mails = self.tx_mails.clone();

                tokio::spawn(async move {
                    // fetch ids of initial mails
                    let mut initial_mail_ids = {
                        let mut request = account.client.build();

                        let query = request.query_email();
                        query.arguments().collapse_threads(true);
                        query
                            .filter(jmap_client::email::query::Filter::in_mailbox(
                                selected_mailbox_id,
                            ))
                            .limit(10)
                            .calculate_total(true)
                            .position(0)
                            .sort([
                                jmap_client::email::query::Comparator::received_at().descending()
                            ]);

                        request.send_query_email().await.unwrap()
                    };

                    // fetch initial mails
                    {
                        let mut request = account.client.build();
                        request.get_email().ids(Some(initial_mail_ids.take_ids()));

                        let mut initial_mails = request.send_get_email().await.unwrap();
                        tx_mails.send(initial_mails.take_list()).await.unwrap();
                    }
                });
            }
        }

        None
    }

    pub fn get_mail(&self, selected_mail_idx: usize) -> Option<Email> {
        if let Some(selected_mailbox_id) = &self.selected_mailbox_id {
            if let Some(mails) = self.mails.get(selected_mailbox_id) {
                if let Some(mails) = mails {
                    return Some(mails[selected_mail_idx].clone());
                }
            }
            debug!(
                "Didn't fetch mails from mailbox with id '{}'",
                selected_mailbox_id
            );
        } else {
            debug!("No mailboxes available yet");
        }

        None
    }

    pub fn select_next_mail(&mut self) {
        self.list_state.next();
    }

    pub fn select_previous_mail(&mut self) {
        self.list_state.previous();
    }

    pub fn selected_mail_list_idx(&self) -> Option<usize> {
        self.list_state.selected
    }

    pub fn get_mail_list_state_mut(&mut self) -> &mut tui_widget_list::ListState {
        &mut self.list_state
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
