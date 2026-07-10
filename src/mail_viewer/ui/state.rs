use super::Action;
use crate::{
    backend::Account,
    mail_viewer::ui::widget::RenderData,
    utils::ui::{MailId, ScreenPalette, ScreenState, keybindmanager::KeybindManager, palette},
};
use chrono::DateTime;
use jmap_client::email::{Email, EmailAddress};
use ratatui::widgets::ScrollbarState;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum PaletteType {
    /// Palette is displaying commands
    Action(Action),
}

pub struct State {
    app_actions: Vec<crate::Action>,
    account: Arc<Account>,

    rx: mpsc::Receiver<Email>,
    tx: Arc<mpsc::Sender<Email>>,

    ctx: Option<Ctx>,
    palette: Option<palette::State<PaletteType>>,
    keybindings: KeybindManager<Action>,
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        let (tx, rx) = mpsc::channel(1);

        Self {
            app_actions: vec![],
            rx,
            tx: Arc::new(tx),
            account,

            ctx: None,
            palette: None,
            keybindings: KeybindManager::new(HashMap::from([
                ("h", Action::ScrollLeft),
                ("j", Action::ScrollDown),
                ("k", Action::ScrollUp),
                ("l", Action::ScrollRight),
                ("q", Action::Quit),
                // ("<BS>", super::Action::OpenMailList(None)),
            ])),
        }
    }

    pub fn open_mail(&mut self, mail: Option<MailId>) {
        if let Some(id) = mail {
            self.ctx = None;
            let account = self.account.clone();
            let tx = self.tx.clone();

            tokio::spawn(async move {
                let client = &account.client;

                let mut response = {
                    let mut request = client.build();
                    request
                        .get_email()
                        .ids(Some([id]))
                        .arguments()
                        .fetch_text_body_values(true);
                    request.send_get_email().await.unwrap()
                };

                tx.send(response.take_list()[0].clone()).await.unwrap();
            });
        }
    }

    fn scroll_down(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_down());
    }

    fn scroll_up(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_up());
    }

    fn scroll_left(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_left());
    }

    fn scroll_right(&mut self) {
        self.ctx.as_mut().map(|ctx| ctx.scroll_right());
    }

    pub fn get_render_data(&mut self) -> Option<RenderData<'_>> {
        self.ctx.as_mut().map(|ctx| ctx.render_data())
    }
}

impl ScreenState<Action, PaletteType> for State {
    fn update(&mut self) {}

    fn apply_action(&mut self, action: Action) {
        debug!("Action: {}", action);
        match action {
            Action::Quit => self.app_actions.push(crate::Action::Quit),
            Action::OpenCommandPalette => {
                self.palette = Some(palette::State::new(super::action::palette_options()));
            }
            Action::CloseCommandPalette => self.palette = None,

            Action::ScrollUp => self.scroll_up(),
            Action::ScrollDown => self.scroll_down(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ScrollRight => self.scroll_right(),

            Action::OpenMailList => {
                todo!()
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

#[derive(Debug)]
struct Ctx {
    pub mail: Email,
    /// String representation of mail
    pub mail_str: String,

    pub vertical: ScrollbarState,
    pub horizontal: ScrollbarState,
}

impl Ctx {
    pub fn new(mail: Email) -> Self {
        let mail_str = Self::get_string_representation(&mail);

        let vertical = ScrollbarState::new(mail_str.lines().count());
        let horizontal =
            ScrollbarState::new(mail_str.lines().map(|line| line.len()).max().unwrap());

        Self {
            mail,
            mail_str,
            vertical,
            horizontal,
        }
    }

    pub fn render_data(&mut self) -> RenderData<'_> {
        RenderData {
            mail: &self.mail,
            mail_str: self.mail_str.as_str(),

            vertical: &mut self.vertical,
            horizontal: &mut self.horizontal,
        }
    }

    pub fn scroll_down(&mut self) {
        self.vertical.next();
    }

    pub fn scroll_up(&mut self) {
        self.vertical.prev();
    }

    pub fn scroll_right(&mut self) {
        self.horizontal.next();
    }

    pub fn scroll_left(&mut self) {
        self.horizontal.prev();
    }

    fn get_string_representation(mail: &Email) -> String {
        let date = DateTime::from_timestamp_millis(mail.received_at().unwrap())
            .unwrap()
            .format("%A, %d %B %Y %T");

        let from = Self::addresses_to_string(mail.from().unwrap());
        let to = Self::addresses_to_string(mail.to().unwrap());
        let subject = mail.subject().unwrap();

        let body = {
            let mut s = String::new();

            for body in mail.text_body().unwrap() {
                let id = body.part_id().unwrap();

                s.push_str(mail.body_value(id).unwrap().value());
            }

            s
        };

        format!(
            "\
Date: {date}
From: {from}
To: {to}
Subject: {subject}


{body}"
        )
    }

    fn addresses_to_string(addresses: &[EmailAddress]) -> String {
        let mut iter = addresses.iter();

        let mut s = format!("{}", iter.next().unwrap().email());

        for addr in iter {
            s.push_str(&format!(", {}", addr.email()))
        }

        s
    }
}
