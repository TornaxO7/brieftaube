use crate::{
    datasource::{
        MailCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailData, MailDataAttachment, MailDataHtmlBody, MailDataTextBody, MailId},
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

    async fn upsert_mails(&mut self, mails: Vec<MailData>) -> Result<(), Self::Error> {
        for mail in mails.into_iter() {
            let id = mail.id.clone();
            self.mails.insert(id, mail);
        }
        Ok(())
    }

    async fn get_mail_text_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataTextBody>, Self::Error> {
        let body = self.mail_text_body.get(id).cloned();
        Ok(body)
    }

    async fn get_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<(MailId, MailDataTextBody)>, Vec<MailId>>, Self::Error>
    {
        let mut cached_text_bodies = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_text_body.get(id) {
                Some(text_body) => cached_text_bodies.push((id.clone(), text_body.clone())),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_text_bodies,
            missing,
        })
    }

    async fn upsert_mail_text_body(
        &mut self,
        id: &MailId,
        body: MailDataTextBody,
    ) -> Result<(), Self::Error> {
        self.mail_text_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mails_text_body(
        &mut self,
        text_bodies: &[(MailId, MailDataTextBody)],
    ) -> Result<(), Self::Error> {
        for (id, text_body) in text_bodies {
            self.mail_text_body.insert(id.clone(), text_body.clone());
        }
        Ok(())
    }

    async fn get_mail_html_body(
        &self,
        id: &MailId,
    ) -> Result<Option<MailDataHtmlBody>, Self::Error> {
        let body = self.mail_html_body.get(id).cloned();
        Ok(body)
    }

    async fn get_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<(MailId, MailDataHtmlBody)>, Vec<MailId>>, Self::Error>
    {
        let mut cached_html_bodies = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_html_body.get(id) {
                Some(html_body) => cached_html_bodies.push((id.clone(), html_body.clone())),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_html_bodies,
            missing,
        })
    }

    async fn upsert_mail_html_body(
        &mut self,
        id: &MailId,
        body: MailDataHtmlBody,
    ) -> Result<(), Self::Error> {
        self.mail_html_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mails_html_body(
        &mut self,
        html_bodies: &[(MailId, MailDataHtmlBody)],
    ) -> Result<(), Self::Error> {
        for (id, html_body) in html_bodies {
            self.mail_html_body.insert(id.clone(), html_body.clone());
        }
        Ok(())
    }

    async fn get_mail_attachments(
        &self,
        id: &MailId,
    ) -> Result<Option<Vec<MailDataAttachment>>, Self::Error> {
        let attachments = self.mail_attachments.get(id).cloned();
        Ok(attachments)
    }

    async fn get_mails_attachments(
        &self,
        ids: &[MailId],
    ) -> Result<
        cache::GetBatchResult<Vec<(MailId, Vec<MailDataAttachment>)>, Vec<MailId>>,
        Self::Error,
    > {
        let mut cached_attachments = Vec::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_attachments.get(id) {
                Some(attachments) => cached_attachments.push((id.clone(), attachments.clone())),
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_attachments,
            missing,
        })
    }

    async fn upsert_mail_attachments(
        &mut self,
        id: &MailId,
        attachments: Vec<MailDataAttachment>,
    ) -> Result<(), Self::Error> {
        self.mail_attachments.insert(id.clone(), attachments);
        Ok(())
    }

    async fn upsert_mails_attachments(
        &mut self,
        attachments: &[(MailId, Vec<MailDataAttachment>)],
    ) -> Result<(), Self::Error> {
        for (id, mail_attachments) in attachments {
            self.mail_attachments
                .insert(id.clone(), mail_attachments.clone());
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
}
