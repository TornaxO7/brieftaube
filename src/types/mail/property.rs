#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MailProperty {
    Id,
    Keywords,
    MailboxIds,
    Subject,
    ReceivedAt,
    HasAttachment,
    ThreadId,

    From,
    To,
    Cc,
    Bcc,
    Preview,
    Attachments,

    TextBody,
    HtmlBody,
}

impl From<jmap_client::email::Property> for MailProperty {
    fn from(property: jmap_client::email::Property) -> Self {
        match property {
            jmap_client::email::Property::Id => Self::Id,
            jmap_client::email::Property::Keywords => Self::Keywords,
            jmap_client::email::Property::MailboxIds => Self::MailboxIds,
            jmap_client::email::Property::Subject => Self::Subject,
            jmap_client::email::Property::ReceivedAt => Self::ReceivedAt,
            jmap_client::email::Property::HasAttachment => Self::HasAttachment,
            jmap_client::email::Property::ThreadId => Self::ThreadId,

            jmap_client::email::Property::From => Self::From,
            jmap_client::email::Property::To => Self::To,
            jmap_client::email::Property::Cc => Self::Cc,
            jmap_client::email::Property::Bcc => Self::Bcc,
            jmap_client::email::Property::Preview => Self::Preview,
            jmap_client::email::Property::Attachments => Self::Attachments,

            jmap_client::email::Property::TextBody => Self::TextBody,
            jmap_client::email::Property::HtmlBody => Self::HtmlBody,

            _other => unreachable!("Not defined for {:?}", _other),
        }
    }
}

impl Into<jmap_client::email::Property> for MailProperty {
    fn into(self) -> jmap_client::email::Property {
        match self {
            MailProperty::Id => jmap_client::email::Property::Id,
            MailProperty::Keywords => jmap_client::email::Property::Keywords,
            MailProperty::MailboxIds => jmap_client::email::Property::MailboxIds,
            MailProperty::Subject => jmap_client::email::Property::Subject,
            MailProperty::ReceivedAt => jmap_client::email::Property::ReceivedAt,
            MailProperty::HasAttachment => jmap_client::email::Property::HasAttachment,
            MailProperty::ThreadId => jmap_client::email::Property::ThreadId,
            MailProperty::From => jmap_client::email::Property::From,
            MailProperty::To => jmap_client::email::Property::To,
            MailProperty::Cc => jmap_client::email::Property::Cc,
            MailProperty::Bcc => jmap_client::email::Property::Bcc,
            MailProperty::Preview => jmap_client::email::Property::Preview,
            MailProperty::Attachments => jmap_client::email::Property::Attachments,
            MailProperty::TextBody => jmap_client::email::Property::TextBody,
            MailProperty::HtmlBody => jmap_client::email::Property::HtmlBody,
        }
    }
}
