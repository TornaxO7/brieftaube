use super::MailboxId;

#[derive(Debug, Clone)]
pub enum Children<T: std::fmt::Debug + Clone> {
    This,
    Child(MailboxId, T),
}
