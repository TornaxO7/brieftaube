use super::{DataCollection, MailData};

pub struct MailIterator<'a> {}

impl<'a> Iterator for MailIterator<'a> {
    type Item = &'a MailData;

    fn next(&mut self) -> Option<Self::Item> {}
}
