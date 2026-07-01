use crate::backend;
use jmap_client::mailbox::Mailbox;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct State {
    account: Arc<backend::Account>,

    rx_mailboxes: mpsc::Receiver<Vec<Mailbox>>,
    mailboxes: Option<Vec<Mailbox>>,

    mailbox_list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx_mailboxes, rx_mailboxes) = mpsc::channel(1);

        let account2 = account.clone();
        tokio::spawn(async move {
            let mut response = {
                let mut request = account2.client.build();
                request.get_mailbox().ids::<[_; 0], String>(None);
                request.send_get_mailbox().await.unwrap()
            };

            tx_mailboxes.send(response.take_list()).await.unwrap();
        });

        Self {
            account,
            mailboxes: None,
            rx_mailboxes,

            mailbox_list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn get_mailboxes(&mut self) -> Option<Vec<Mailbox>> {
        if let Some(mailboxes) = self.mailboxes.clone() {
            Some(mailboxes)
        } else {
            match self.rx_mailboxes.try_recv() {
                Ok(mailboxes) => {
                    self.mailboxes = Some(mailboxes);
                    return self.mailboxes.clone();
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
            }

            None
        }
    }

    pub fn get_mut_selected_mailbox(&mut self) -> Option<&mut Mailbox> {
        let Some(mailboxes) = &mut self.mailboxes else {
            return None;
        };

        let Some(idx) = self.mailbox_list_state.selected else {
            return None;
        };

        mailboxes.get_mut(idx)
    }

    pub fn get_list_state(&mut self) -> &mut tui_widget_list::ListState {
        &mut self.mailbox_list_state
    }

    pub fn select_next_mailbox(&mut self) {
        self.mailbox_list_state.next();
    }

    pub fn select_previous_mailbox(&mut self) {
        self.mailbox_list_state.previous();
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish()
    }
}
