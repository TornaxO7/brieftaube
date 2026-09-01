use crate::{
    datasource::{
        MailCache,
        hashmap::{HashMapDataSource, utils::root_mails::RootMails},
        types::{GetState, QueryState, QueryWindow, cache},
    },
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
};

impl MailCache for HashMapDataSource {
    async fn get_mail_state(&self) -> Option<&GetState> {
        self.mail_get_state.as_ref()
    }

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<(), Self::Error> {
        self.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<MailData>, Vec<MailId>>, Self::Error> {
        let mut values = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mails.get(id) {
                Some(data) => values.push(data.clone()),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: values,
            missing,
        })
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataTextBody>, Self::Error> {
        let body = self.mail_text_body.get(id).cloned();
        Ok(body)
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataHtmlBody>, Self::Error> {
        let body = self.mail_html_body.get(id).cloned();
        Ok(body)
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Option<Vec<MailDataAttachment>>, Self::Error> {
        let attachments = self.mail_attachments.get(id).cloned();
        Ok(attachments)
    }

    async fn get_root_mails_state(&self, mailbox: &MailboxId) -> Option<&QueryState> {
        self.root_mails
            .get(mailbox)
            .map(|root_mails| root_mails.state())
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<Option<cache::QueryResponse<MailData>>, Self::Error> {
        let range = window.as_range();
        let Some(root_mails) = self.root_mails.get(mailbox) else {
            return Ok(None);
        };

        Ok(Some(root_mails.query(range).map(|id| {
            self.mails
                .get(&id)
                .cloned()
                .expect("MailData has been fetched as well")
        })))
    }

    async fn upsert_mails(&mut self, mails: Vec<MailData>) -> Result<(), Self::Error> {
        for mail in mails.into_iter() {
            let id = mail.id.clone();
            self.mails.insert(id, mail);
        }
        Ok(())
    }

    async fn evict_mails(&mut self, mails: &[MailId]) -> Result<(), Self::Error> {
        for id in mails {
            self.mail_text_body.remove(id);
            self.mail_html_body.remove(id);
            self.mail_attachments.remove(id);

            if let Some(mail) = self.mails.remove(id) {
                for mailbox in mail.mailbox_ids {
                    // removing leads to position changes, mail changes (in a thread) etc.
                    // => Just clear it.
                    // TODO: Maybe... try first to use the data from the cache (thread check etc.)
                    if let Some(root_mails) = self.root_mails.get_mut(&mailbox) {
                        root_mails.flush();
                    }
                }
            }
        }
        Ok(())
    }

    async fn upsert_root_mails(
        &mut self,
        id: &MailboxId,
        start: usize,
        root_mails: Vec<MailData>,
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        let root_mail_ids = root_mails.iter().map(|data| data.id.clone()).collect();

        match self.root_mails.entry(id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let root_mails = entry.get_mut();
                root_mails.set(start, root_mail_ids, new_state);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(RootMails::new(start, root_mail_ids, new_state));
            }
        }

        for root_mail in root_mails {
            let id = root_mail.id.clone();
            self.mails.insert(id, root_mail);
        }

        Ok(())
    }

    async fn upsert_mail_text_body(
        &mut self,
        id: &MailId,
        body: MailDataTextBody,
    ) -> Result<(), Self::Error> {
        self.mail_text_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mail_html_body(
        &mut self,
        id: &MailId,
        body: MailDataHtmlBody,
    ) -> Result<(), Self::Error> {
        self.mail_html_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mail_attachments(
        &mut self,
        id: &MailId,
        attachments: Vec<MailDataAttachment>,
    ) -> Result<(), Self::Error> {
        self.mail_attachments.insert(id.clone(), attachments);
        Ok(())
    }
}
