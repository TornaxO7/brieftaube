use super::action::Action;
use crate::{
    backend::Account,
    utils::ui::{MailboxId, ScreenPalette, ScreenState, keybindmanager::KeybindManager, palette},
};
use chrono::Local;
use jmap_client::email::{EmailBodyPart, EmailBodyValue};
use mail_parser::MessageParser;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    account: Arc<Account>,

    raw_mail: String,

    rx: mpsc::Receiver<MailboxId>,
    _tx: Arc<mpsc::Sender<MailboxId>>,
    draft_mailbox_id: Option<MailboxId>,

    scroll_offset: (u16, u16),

    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<Action>,
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        let acc = account.clone();
        let tx = Arc::new(tx);
        let tx2 = tx.clone();

        tokio::spawn(async move {
            let mut request = acc.client.build();
            request.get_mailbox().ids(None::<[String; 0]>);
            let response = request.send_get_mailbox().await.unwrap();

            let sent_mailbox = response
                .list()
                .iter()
                .find(|mailbox| mailbox.role() == jmap_client::mailbox::Role::Drafts)
                .unwrap();

            tx2.send(sent_mailbox.id().unwrap().to_string())
                .await
                .unwrap();
        });

        let mut state = Self {
            app_actions: vec![],
            account: account.clone(),
            raw_mail: String::new(),
            scroll_offset: (0, 0),
            draft_mailbox_id: None,
            rx,
            _tx: tx,

            palette: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                // ("q", super::Action::Quit),
                // ("h", super::Action::OpenMailList(None)),
                ("e", Action::OpenMailInEditor),
                (":", Action::OpenCommandPalette),
            ])),
        };

        state.reset();
        state
    }

    pub fn reset(&mut self) {
        let address = self.account.address();

        self.raw_mail = format!(
            "\
From: {address}
To:
Subject:

"
        );
    }

    fn scroll_down(&mut self) {
        self.scroll_offset.0 += 1;
    }

    fn scroll_up(&mut self) {
        self.scroll_offset.0 = self.scroll_offset.0.saturating_sub(1);
    }

    pub fn get_mail(&self) -> &str {
        self.raw_mail.as_str()
    }

    fn open_mail_in_editor(&mut self) {
        let tmp_file = get_tmp_file();

        std::fs::write(&tmp_file, &self.raw_mail).unwrap();
        std::process::Command::new("hx")
            .arg(&tmp_file)
            .output()
            .unwrap();

        // TODO: Check if the changed mail is correct
        self.raw_mail = std::fs::read_to_string(&tmp_file).unwrap();
    }

    pub fn send_mail(&mut self) {
        if let Some(draft_id) = self.draft_mailbox_id.clone() {
            let account = self.account.clone();
            let raw_mail = self.raw_mail.clone();

            tokio::spawn(async move {
                let parsed = MessageParser::new().parse(raw_mail.as_bytes()).unwrap();

                let mut request = account.client.build();

                request
                    .set_email()
                    .create()
                    .sent_at(Local::now().timestamp())
                    .from(mail_parser_address_to_jmap_client_address(
                        parsed.from().unwrap(),
                    ))
                    .to(mail_parser_address_to_jmap_client_address(
                        parsed.to().unwrap(),
                    ))
                    .subject(parsed.subject().unwrap())
                    .mailbox_id(&draft_id, true)
                    .body_value(
                        "1".to_string(),
                        EmailBodyValue::from(parsed.body_text(0).unwrap().to_string()),
                    )
                    .text_body(EmailBodyPart::new().part_id("1").content_type("text/plain"));

                request.send_set_email().await.unwrap();
            });
        }
    }
}

impl ScreenState<Action, PaletteType> for State {
    fn update(&mut self) {
        match self.rx.try_recv() {
            Ok(sent_mailbox_id) => self.draft_mailbox_id = Some(sent_mailbox_id),
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => todo!(),
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(super::action::palette_options()));
            }
            Action::CloseCommandPalette => self.palette = None,
            Action::ScrollUp => self.scroll_up(),
            Action::ScrollDown => self.scroll_down(),

            Action::OpenMailList => {
                todo!()
            }
            Action::OpenMailInEditor => self.open_mail_in_editor(),
            Action::SendMail => {
                self.send_mail();
                self.reset();
                todo!()
                // return Some(super::Action::OpenMailboxList);
            }
        }
    }

    fn get_app_actions(&mut self) -> std::vec::Drain<'_, crate::Action> {
        self.app_actions.drain(..)
    }

    fn keybinding_manager(&mut self) -> &mut KeybindManager<Action> {
        &mut self.keybindings
    }
}

impl ScreenPalette<PaletteType> for State {
    fn palette(&mut self) -> Option<&mut palette::State<PaletteType>> {
        self.palette.as_mut()
    }

    fn handle_palette_result(&mut self, result: palette::HandleEventResult<PaletteType>) {
        match result {
            palette::HandleEventResult::Cancel => {}
            palette::HandleEventResult::Selected(value) => match value {
                PaletteType::Action(action) => self.apply_action(action),
            },
        }
    }
}

fn get_tmp_file() -> PathBuf {
    let xdg = crate::get_xdg();

    xdg.place_cache_file("temp.mail").unwrap()
}

fn mail_parser_address_to_jmap_client_address(
    addr: &mail_parser::Address,
) -> Vec<jmap_client::email::EmailAddress> {
    match addr {
        mail_parser::Address::List(list) => list
            .iter()
            .map(|addr| match addr.name() {
                Some(name) => {
                    jmap_client::email::EmailAddress::from((name, addr.address().unwrap()))
                }
                None => jmap_client::email::EmailAddress::from(addr.address().unwrap()),
            })
            .collect(),
        mail_parser::Address::Group(_group) => todo!(),
    }
}
