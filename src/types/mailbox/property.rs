#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MailboxProperty {
    Id,
    Name,
    Role,
    SortOrder,
    ParentId,
    UnreadEmails,
    UnreadThreads,
    TotalThreads,
    MyRights,
}

impl From<jmap_client::mailbox::Property> for MailboxProperty {
    fn from(property: jmap_client::mailbox::Property) -> Self {
        match property {
            jmap_client::mailbox::Property::Id => Self::Id,
            jmap_client::mailbox::Property::Name => Self::Name,
            jmap_client::mailbox::Property::Role => Self::Role,
            jmap_client::mailbox::Property::SortOrder => Self::SortOrder,
            jmap_client::mailbox::Property::ParentId => Self::ParentId,
            jmap_client::mailbox::Property::UnreadEmails => Self::UnreadEmails,
            jmap_client::mailbox::Property::UnreadThreads => Self::UnreadThreads,
            jmap_client::mailbox::Property::TotalThreads => Self::TotalThreads,
            jmap_client::mailbox::Property::MyRights => Self::MyRights,
            other => unreachable!("Not defined for {:?}", other),
        }
    }
}

impl Into<jmap_client::mailbox::Property> for MailboxProperty {
    fn into(self) -> jmap_client::mailbox::Property {
        match self {
            MailboxProperty::Id => jmap_client::mailbox::Property::Id,
            MailboxProperty::Name => jmap_client::mailbox::Property::Name,
            MailboxProperty::Role => jmap_client::mailbox::Property::Role,
            MailboxProperty::SortOrder => jmap_client::mailbox::Property::SortOrder,
            MailboxProperty::ParentId => jmap_client::mailbox::Property::ParentId,
            MailboxProperty::UnreadEmails => jmap_client::mailbox::Property::UnreadEmails,
            MailboxProperty::UnreadThreads => jmap_client::mailbox::Property::UnreadThreads,
            MailboxProperty::TotalThreads => jmap_client::mailbox::Property::TotalThreads,
            MailboxProperty::MyRights => jmap_client::mailbox::Property::MyRights,
        }
    }
}
