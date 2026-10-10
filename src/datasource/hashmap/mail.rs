use crate::{
    datasource::{
        MailCache,
        hashmap::HashMapDataSource,
        types::{GetState, cache},
    },
    types::{CachedMail, MailId},
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

    async fn get_mails(
        &self,
        ids: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<CachedMail>, Vec<MailId>>> {
        let mut datas = Vec::with_capacity(ids.len());
        let mut missing = Vec::new();

        for id in ids {
            match self.mails.get(id) {
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

    async fn evict_mails(
        &mut self,
        mails: &[MailId],
    ) -> Result<cache::GetBatchResult<Vec<CachedMail>, Vec<MailId>>> {
        let mut missing = Vec::new();
        let mut removed_mails = Vec::new();

        for id in mails {
            match self.mails.remove(id) {
                Some(removed_mail) => {
                    if let Some(mailbox_ids) = removed_mail.mailbox_ids.get() {
                        for mailbox in mailbox_ids {
                            // removing leads to position changes, mail changes (in a thread) etc.
                            // => Just clear it.
                            // TODO: Maybe... try first to use the data from the cache (thread check etc.)
                            if let Some(root_mails) = self.root_mails.get_mut(&mailbox) {
                                root_mails.flush();
                            }
                        }
                    }

                    removed_mails.push(removed_mail);
                }
                None => missing.push(id.clone()),
            }
        }

        Ok(cache::GetBatchResult {
            value: removed_mails,
            missing,
        })
    }

    async fn upsert_mails(&mut self, mails: Vec<CachedMail>) -> Result<()> {
        for mail in mails {
            self.mails.insert(mail.id.clone(), mail);
        }

        Ok(())
    }
}
