use crate::{
    types::{MailDataAttachment, MailDataHtmlBody, MailDataTextBody},
    ui::reader::ReaderHeaders,
};
use crossterm::event::Event;

pub enum Message {
    UserAction(super::UserAction),
    Event(Event),
    SelectedPaletteEntry(String),

    SetHeaders(ReaderHeaders),
    SetTextBody(MailDataTextBody),
    SetHtmlBody(MailDataHtmlBody),
    SetAttachments(Vec<MailDataAttachment>),

    Reset,
}
