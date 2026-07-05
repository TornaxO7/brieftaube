use crate::backend;
use jmap_client::mailbox::Mailbox;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct State {
    _account: Arc<backend::Account>,

    _tx: Arc<mpsc::Sender<Vec<Mailbox>>>,
    rx: mpsc::Receiver<Vec<Mailbox>>,
    mailboxes: Option<Vec<Mailbox>>,

    mailbox_list_state: tui_widget_list::ListState,
}

impl State {
    pub fn new(account: Arc<backend::Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let tx = Arc::new(tx);

        let tx2 = tx.clone();
        let account2 = account.clone();
        tokio::spawn(async move {
            let mut response = {
                let mut request = account2.client.build();
                request
                    .get_mailbox()
                    .ids::<[_; 0], String>(None)
                    .properties([
                        jmap_client::mailbox::Property::Id,
                        jmap_client::mailbox::Property::Name,
                        jmap_client::mailbox::Property::Role,
                        jmap_client::mailbox::Property::TotalEmails,
                        jmap_client::mailbox::Property::UnreadEmails,
                    ]);
                request.send_get_mailbox().await.unwrap()
            };

            tx2.send(response.take_list()).await.unwrap();
        });

        Self {
            _account: account,
            mailboxes: None,
            rx,
            _tx: tx,

            mailbox_list_state: tui_widget_list::ListState::default(),
        }
    }

    pub fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(mailboxes) => self.mailboxes = Some(mailboxes),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }

        let select_first_entry =
            self.mailboxes.is_some() && self.mailbox_list_state.selected.is_none();

        if select_first_entry {
            self.mailbox_list_state.next();
        }
    }

    pub fn get_mailboxes(&mut self) -> Option<Vec<Mailbox>> {
        self.mailboxes.clone()
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
