use crate::{backend::Account, ui::composer::mail_template::MailTemplate};
use mail_parser::MessageParser;
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct State {
    account: Arc<Account>,

    mail: MailTemplate,
    scroll_offset: (u16, u16),
}

impl State {
    pub fn new(account: Arc<Account>) -> Self {
        Self {
            account: account.clone(),
            mail: MailTemplate::new(account.clone()),
            scroll_offset: (0, 0),
        }
    }

    pub fn reset(&mut self) {
        self.mail = MailTemplate::new(self.account.clone());
    }

    pub fn update(&mut self) {}

    pub fn scroll_down(&mut self) {
        self.scroll_offset.0 += 1;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset.0 = self.scroll_offset.0.saturating_sub(1);
    }

    pub fn get_mail(&self) -> &MailTemplate {
        &self.mail
    }

    pub fn open_mail_in_editor(&mut self) {
        let content = self.mail.renderable();

        let tmp_file = get_tmp_file();

        std::fs::write(&tmp_file, content).unwrap();
        std::process::Command::new("hx")
            .arg(&tmp_file)
            .output()
            .unwrap();

        let content = std::fs::read_to_string(&tmp_file).unwrap();
        let message = MessageParser::default().parse(content.as_bytes()).unwrap();
        tracing::debug!("{:#?}", message);

        self.mail.to = message.header_raw("to").unwrap().trim().to_string();
        self.mail.subject = message.header_raw("subject").unwrap().trim().to_string();
        self.mail.from = message.header_raw("from").unwrap().trim().to_string();
        self.mail.body = message.body_text(0).unwrap().to_string();
    }
}

// TODO: Create directory which the user can use to just copy the attachment files to
fn get_tmp_file() -> PathBuf {
    let xdg = crate::get_xdg();

    xdg.place_cache_file("temp.mail").unwrap()
}
