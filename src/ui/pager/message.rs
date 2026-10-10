use crate::{
    types::{CachedAttachment, MailDataHtmlBody, MailDataTextBody},
    ui::{pager::MailHeaders, statusbar::StatusMsgType},
};
use crossterm::event::Event;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),

    SetHeaders(color_eyre::Result<MailHeaders>),
    SetTextBody(color_eyre::Result<MailDataTextBody>),
    SetHtmlBody(color_eyre::Result<MailDataHtmlBody>),
    SetAttachments(color_eyre::Result<Vec<CachedAttachment>>),

    SetStatusbarMessage { msg: String, ty: StatusMsgType },

    SaveAttachment(SaveAttachmentStep),
}

#[derive(Debug)]
pub enum SaveAttachmentStep {
    GetDestinationPath(color_eyre::Result<Vec<u8>>),
    SaveAttachment { blob: Vec<u8>, path: PathBuf },
}

impl From<Message> for crate::ui::Message {
    fn from(msg: Message) -> Self {
        Self::Pager(msg)
    }
}
