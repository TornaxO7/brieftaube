use crate::{
    config::Username,
    types::{AccountId, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId},
    ui::reader::ReaderHeaders,
};
use crossterm::event::Event;
use ratatui::layout::Size;

pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),

    SetMailBodySize(Size),

    SetHeaders(color_eyre::Result<ReaderHeaders>),
    SetTextBody(color_eyre::Result<MailDataTextBody>),
    SetHtmlBody(color_eyre::Result<MailDataHtmlBody>),
    SetAttachments(color_eyre::Result<Vec<MailDataAttachment>>),

    Reset {
        username: Username,
        account_id: AccountId,
        mail_id: MailId,
    },
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Reader(msg)
    }
}
