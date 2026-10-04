use crate::{
    types::{MailDataAttachment, MailDataHtmlBody, MailDataTextBody},
    ui::{pager::MailHeaders, statusbar::StatusMsgType},
};
use crossterm::event::Event;

#[derive(Debug)]
pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),

    SetHeaders(color_eyre::Result<MailHeaders>),
    SetTextBody(color_eyre::Result<MailDataTextBody>),
    SetHtmlBody(color_eyre::Result<MailDataHtmlBody>),
    SetAttachments(color_eyre::Result<Vec<MailDataAttachment>>),

    SetStatusbarMessage { msg: String, ty: StatusMsgType },
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Pager(msg)
    }
}
