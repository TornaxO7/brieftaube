use crate::{
    datasources::{
        MailCache,
        hashmap::{HashMapDataSource, utils::root_mails::RootMails},
        types::{GetState, QueryState, QueryWindow, cache},
    },
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId, MailboxId},
};

impl MailCache for HashMapDataSource {
    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetResult<Vec<Option<MailData>>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let datas = ids.iter().map(|id| inner.mails.get(id).cloned()).collect();

        Ok(cache::GetResult {
            value: datas,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<MailDataTextBody>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let body = inner.mail_text_body.get(id).cloned();

        Ok(cache::GetResult {
            value: body,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<MailDataHtmlBody>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let body = inner.mail_html_body.get(id).cloned();

        Ok(cache::GetResult {
            value: body,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<cache::GetResult<Option<Vec<MailDataAttachment>>>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let attachments = inner.mail_attachments.get(id).cloned();

        Ok(cache::GetResult {
            value: attachments,
            state: inner.mail_get_state.clone(),
        })
    }

    async fn query_root_mails(
        &self,
        mailbox: &MailboxId,
        window: QueryWindow,
    ) -> Result<cache::QueryResponse<MailId>, Self::Error> {
        let inner = self.inner.read().unwrap();
        let range = window.as_range();

        match inner.root_mails.get(mailbox) {
            None => Ok(cache::QueryResponse {
                values: vec![],
                missing: vec![range],
                query_state: None,
            }),
            Some(root_mails) => Ok(root_mails.query(range)),
        }
    }

    async fn upsert_mails(
        &self,
        mails: Vec<MailData>,
        new_state: GetState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for mail in mails.into_iter() {
            let id = mail.id.clone();
            inner.mails.insert(id, mail);
        }

        inner.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn evict_mails(&self, mails: &[MailId], new_state: GetState) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        for id in mails {
            inner.mail_text_body.remove(id);
            inner.mail_html_body.remove(id);
            inner.mail_attachments.remove(id);

            if let Some(mail) = inner.mails.remove(id) {
                for mailbox in mail.mailbox_ids {
                    // removing leads to position changes, mail changes (in a thread) etc.
                    // => Just clear it.
                    // TODO: Maybe... try first to use the data from the cache (thread check etc.)
                    if let Some(root_mails) = inner.root_mails.get_mut(&mailbox) {
                        root_mails.flush();
                    }
                }
            }
        }

        inner.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn upsert_root_mails(
        &self,
        id: &MailboxId,
        start: usize,
        ids: Vec<MailId>,
        new_state: QueryState,
    ) -> Result<(), Self::Error> {
        let mut inner = self.inner.write().unwrap();

        match inner.root_mails.entry(id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let root_mails = entry.get_mut();
                root_mails.set(start, ids, new_state);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(RootMails::new(start, ids, new_state));
            }
        }

        Ok(())
    }
}
