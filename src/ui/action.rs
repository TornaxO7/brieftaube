use super::*;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,

    MailboxList(mailboxes::Action),
    MailList(mails::Action),
    MailViewer(mail_viewer::Action),
    Composer(composer::Action),
    LogViewer(log_viewer::Action),

    OpenMailList(Option<MailboxId>),
    OpenMailViewer(Option<MailId>),
    OpenMailboxList,
    OpenComposer,
    OpenLogs(Box<Self>),
}

impl From<mails::Action> for Action {
    fn from(action: mails::Action) -> Self {
        Self::MailList(action)
    }
}

impl From<mailboxes::Action> for Action {
    fn from(action: mailboxes::Action) -> Self {
        Self::MailboxList(action)
    }
}

impl From<mail_viewer::Action> for Action {
    fn from(action: mail_viewer::Action) -> Self {
        Self::MailViewer(action)
    }
}

impl From<composer::Action> for Action {
    fn from(action: composer::Action) -> Self {
        Self::Composer(action)
    }
}

impl From<log_viewer::Action> for Action {
    fn from(action: log_viewer::Action) -> Self {
        Self::LogViewer(action)
    }
}
