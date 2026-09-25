use crate::{
    datasource::{
        MailCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{MailDataCore, MailDataHtmlBody, MailDataPreview, MailDataTextBody, MailId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl MailCache for HashMapDataSource {
    async fn get_mail_state(&self) -> Option<&GetState> {
        self.mail_get_state.as_ref()
    }

    async fn set_mail_state(&mut self, new_state: GetState) -> Result<()> {
        self.mail_get_state = Some(new_state);
        Ok(())
    }

    async fn get_mails_core(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<MailDataCore>, Vec<MailId>>> {
        let mut datas = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();

        for id in ids {
            match self.mails_core.get(id) {
                Some(core) => {
                    datas.push(core.clone());
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
    ) -> Result<cache::GetBatchResult<Vec<MailDataPreview>, Vec<MailId>>> {
        let mut datas = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();

        for id in ids {
            match self.mails_preview.get(id) {
                Some(core) => {
                    datas.push(core.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: datas,
            missing,
        })
    }

    async fn get_mails_text_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<MailDataTextBody>, Vec<MailId>>> {
        let mut cached_text_bodies = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_text_body.get(&id) {
                Some(text_body) => {
                    cached_text_bodies.push(text_body.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_text_bodies,
            missing,
        })
    }

    async fn upsert_mail_text_body(&mut self, id: &MailId, body: MailDataTextBody) -> Result<()> {
        self.mail_text_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mails_text_body(&mut self, text_bodies: &[MailDataTextBody]) -> Result<()> {
        for text_body in text_bodies {
            self.mail_text_body
                .insert(text_body.id.clone(), text_body.clone());
        }
        Ok(())
    }

    async fn get_mails_html_body(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<MailDataHtmlBody>, Vec<MailId>>> {
        let mut cached_html_bodies = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();

        for id in ids {
            match self.mail_html_body.get(&id) {
                Some(html_body) => {
                    cached_html_bodies.push(html_body.clone());
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: cached_html_bodies,
            missing,
        })
    }

    async fn upsert_mail_html_body(&mut self, id: &MailId, body: MailDataHtmlBody) -> Result<()> {
        self.mail_html_body.insert(id.clone(), body);
        Ok(())
    }

    async fn upsert_mails_html_body(&mut self, html_bodies: &[MailDataHtmlBody]) -> Result<()> {
        for html_body in html_bodies {
            self.mail_html_body
                .insert(html_body.id.clone(), html_body.clone());
        }
        Ok(())
    }

    async fn evict_mails(&mut self, mails: &[MailId]) -> Result<()> {
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

    async fn upsert_mails_core(&mut self, mails: Vec<MailDataCore>) -> Result<()> {
        for mail in mails {
            self.mails_core.insert(mail.id.clone(), mail);
        }

        Ok(())
    }

    async fn upsert_mails_preview(&mut self, mails: Vec<MailDataPreview>) -> Result<()> {
        for mail in mails {
            self.mails_preview.insert(mail.id.clone(), mail);
        }
        Ok(())
    }
}
