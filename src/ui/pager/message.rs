use crate::{types::CachedMail, ui::statusbar::StatusMsgType};
use crossterm::event::Event;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),

    SetMail(color_eyre::Result<CachedMail>),
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
