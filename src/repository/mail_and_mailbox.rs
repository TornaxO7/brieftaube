use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use crate::{
    datasource::types::{MailPatchType, Patches},
    repository::Repository,
    types::{AccountId, CachedMail, CachedMailbox, MailId, MailKeyword, MailboxId},
};
use tokio::sync::oneshot;
use tracing::error;

#[derive(Debug)]
pub struct Command {
    pub account_id: AccountId,
    pub kind: CommandKind,
}

#[derive(Debug)]
pub enum CommandKind {
    PatchMailAndMailbox {
        patches: Patches,

        tx: oneshot::Sender<color_eyre::Result<()>>,
    },
}

impl From<Command> for super::Command {
    fn from(cmd: Command) -> Self {
        Self::MailAndMailbox(cmd)
    }
}

impl Repository {
    pub async fn patch_mail_and_mailbox(
        &self,
        account_id: AccountId,
        patches: Patches,
    ) -> color_eyre::Result<()> {
        // TODO: HERE
        // Sanitize-Check that there's no mailbox-patch where a mailbox itself should be moved
        // into itself!!!

        match self
            .remote
            .get_remote_account(account_id.clone())
            .patch_mails_and_mailboxes(patches.clone())
            .await
        {
            Ok((mailbox_result, mail_result)) => {
                let mut cache_lock = self.caches.get(&account_id).unwrap().write().await;

                self.ensure_mail_changes(&account_id, &mail_result.new_state, &mut cache_lock)
                    .await?;

                self.ensure_mailbox_changes(
                    &account_id,
                    &mailbox_result.new_state,
                    &mut cache_lock,
                )
                .await?;

                // mailboxes first...
                {
                    let mut impacted_mailboxes = cache_lock
                        .get_mailboxes(&mailbox_result.value.unwrap())
                        .await?
                        .value
                        .into_iter()
                        .map(|mailbox| (mailbox.id.clone(), mailbox))
                        .collect::<HashMap<MailboxId, CachedMailbox>>();

                    for patch in patches.mailbox_patches {
                        let mailbox = impacted_mailboxes.get_mut(&patch.id).unwrap();

                        if let Some(new_name) = patch.new_name {
                            mailbox.name = OnceLock::from(new_name);
                        }

                        if let Some(new_parent_id) = patch.new_parent_id {
                            mailbox.parent_id = OnceLock::from(new_parent_id);
                        }

                        if let Some(new_role) = patch.new_role {
                            mailbox.role = OnceLock::from(new_role);
                        }

                        if let Some(new_sort_order) = patch.new_sort_order {
                            mailbox.sort_order = OnceLock::from(new_sort_order);
                        }
                    }

                    cache_lock
                        .upsert_mailboxes(impacted_mailboxes.into_values().collect())
                        .await?;
                }

                // ...then mails
                {
                    let mut impacted_mails = cache_lock
                        .get_mails(&mail_result.value.unwrap())
                        .await?
                        .value
                        .into_iter()
                        .map(|mail| (mail.id.clone(), mail))
                        .collect::<HashMap<MailId, CachedMail>>();

                    for patch in patches.mail_patches {
                        let mail = impacted_mails.get_mut(&patch.id).unwrap();

                        if let Some(new_mailbox_ids) = patch.new_mailbox_ids {
                            match new_mailbox_ids {
                                MailPatchType::Set(mailbox_ids) => {
                                    mail.mailbox_ids = OnceLock::from(mailbox_ids);
                                }
                                MailPatchType::Patch(changes) => match mail.mailbox_ids.get_mut() {
                                    Some(current_mailbox_ids) => {
                                        for (mailbox_id, set) in changes {
                                            if set {
                                                current_mailbox_ids.insert(mailbox_id);
                                            } else {
                                                current_mailbox_ids.remove(&mailbox_id);
                                            }
                                        }
                                    }
                                    None => {
                                        let new_mailbox_ids: HashSet<MailboxId> = changes
                                            .into_iter()
                                            .filter_map(|(mailbox_id, set)| {
                                                set.then_some(mailbox_id)
                                            })
                                            .collect();
                                        mail.mailbox_ids = OnceLock::from(new_mailbox_ids);
                                    }
                                },
                            }
                        }

                        if let Some(keywords) = patch.new_keywords {
                            match keywords {
                                MailPatchType::Set(keywords) => {
                                    mail.keywords = OnceLock::from(
                                        keywords.into_iter().collect::<HashSet<MailKeyword>>(),
                                    );
                                }
                                MailPatchType::Patch(changes) => match mail.keywords.get_mut() {
                                    Some(current_keywords) => {
                                        for (keyword, set) in changes {
                                            if set {
                                                current_keywords.insert(keyword);
                                            } else {
                                                current_keywords.remove(&keyword);
                                            }
                                        }
                                    }
                                    None => {
                                        let new_keywords: HashSet<MailKeyword> = changes
                                            .into_iter()
                                            .filter_map(|(keyword, set)| set.then_some(keyword))
                                            .collect();
                                        mail.keywords = OnceLock::from(new_keywords);
                                    }
                                },
                            }
                        }
                    }

                    cache_lock
                        .upsert_mails(impacted_mails.into_values().collect())
                        .await?;
                }
                Ok(())
            }
            Err(err) => {
                error!("Couldn't patch mails and mailboxes: {}", err);
                Err(err)
            }
        }
    }
}
