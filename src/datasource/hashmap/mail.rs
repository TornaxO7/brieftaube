use std::collections::HashMap;

use crate::{
    datasource::{
        MailCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailId},
};

impl MailCache for HashMapDataSource {
    async fn get_mail_state(&self) -> Option<&GetState> {
        self.mail_get_state.as_ref()
    }

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<(), Self::Error> {
        self.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn get_mails_core(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataCore>, Vec<MailId>>, Self::Error>
    {
        let mut datas = HashMap::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mails_core.get(id) {
                Some(core) => {
                    datas.insert(id.clone(), core.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: datas,
            missing,
        })
    }

    async fn get_mails_preview(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataPreview>, Vec<MailId>>, Self::Error>
    {
        let mut datas = HashMap::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mails_preview.get(id) {
                Some(core) => {
                    datas.insert(id.clone(), core.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: datas,
            missing,
        })
    }

    async fn get_mails_text_body<MailIds>(
        &self,
        ids: MailIds,
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataTextBody>, Vec<MailId>>, Self::Error>
    where
        MailIds: IntoIterator<Item = MailId>,
    {
        let mut cached_text_bodies = HashMap::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_text_body.get(&id) {
                Some(text_body) => {
                    cached_text_bodies.insert(id.clone(), text_body.clone());
                }
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

    async fn upsert_mails_text_body<MailTextBodies>(
        &mut self,
        text_bodies: MailTextBodies,
    ) -> Result<(), Self::Error>
    where
        MailTextBodies: IntoIterator<Item = (MailId, MailDataTextBody)>,
    {
        for (id, text_body) in text_bodies {
            self.mail_text_body.insert(id.clone(), text_body.clone());
        }
        Ok(())
    }

    async fn get_mails_html_body<MailIds>(
        &self,
        ids: MailIds,
    ) -> Result<cache::GetBatchResult<HashMap<MailId, MailDataHtmlBody>, Vec<MailId>>, Self::Error>
    where
        MailIds: IntoIterator<Item = MailId>,
    {
        let mut cached_html_bodies = HashMap::new();
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_html_body.get(&id) {
                Some(html_body) => {
                    cached_html_bodies.insert(id.clone(), html_body.clone());
                }
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

    async fn upsert_mails_html_body<MailHtmlBodies>(
        &mut self,
        html_bodies: MailHtmlBodies,
    ) -> Result<(), Self::Error>
    where
        MailHtmlBodies: IntoIterator<Item = (MailId, MailDataHtmlBody)>,
    {
        for (id, html_body) in html_bodies {
            self.mail_html_body.insert(id.clone(), html_body.clone());
        }
        Ok(())
    }

    async fn evict_mails<MailIds>(&mut self, mails: MailIds) -> Result<(), Self::Error>
    where
        MailIds: IntoIterator<Item = MailId>,
    {
        for id in mails {
            self.mail_text_body.remove(&id);
            self.mail_html_body.remove(&id);
            self.mails_preview.remove(&id);

            if let Some(mail) = self.mails_core.remove(&id) {
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

    async fn upsert_mails_core<Mails>(&mut self, mails: Mails) -> Result<(), Self::Error>
    where
        Mails: IntoIterator<Item = (MailId, MailDataCore)>,
    {
        for (id, mail) in mails {
            self.mails_core.insert(id, mail);
        }

        Ok(())
    }

    async fn upsert_mails_preview<Mails>(&mut self, mails: Mails) -> Result<(), Self::Error>
    where
        Mails: IntoIterator<Item = (MailId, MailDataPreview)>,
    {
        for (id, mail) in mails {
            self.mails_preview.insert(id, mail);
        }
        Ok(())
    }
}
