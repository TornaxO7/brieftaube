use crate::{
    datasource::{
        MailMailboxRemote,
        jmap::JmapAccount,
        types::{GetState, MailPatchType, Patches, remote},
    },
    types::{MailId, MailboxId},
};
use async_trait::async_trait;
use color_eyre::Result;

#[async_trait]
impl MailMailboxRemote for JmapAccount {
    async fn patch_mails_and_mailboxes(
        &self,
        patches: Patches,
    ) -> Result<(
        remote::SetResult<Vec<MailboxId>>,
        remote::SetResult<Vec<MailId>>,
    )> {
        debug_assert!(!patches.mail_patches.is_empty() || !patches.mailbox_patches.is_empty());

        let mut response = {
            let mut request = self.build_request();

            // collect mail patches
            {
                let set_email = request.set_email();

                for mail_patch in patches.mail_patches {
                    let update = set_email.update(mail_patch.id);

                    if let Some(mailbox_ids) = mail_patch.new_mailbox_ids {
                        match mailbox_ids {
                            MailPatchType::Set(ids) => {
                                update.mailbox_ids(ids);
                            }
                            MailPatchType::Patch(changes) => {
                                for (mailbox_id, set) in changes {
                                    update.mailbox_id(mailbox_id.as_str(), set);
                                }
                            }
                        };
                    }

                    if let Some(keywords) = mail_patch.new_keywords {
                        match keywords {
                            MailPatchType::Set(keywords) => {
                                update.keywords(keywords);
                            }
                            MailPatchType::Patch(changes) => {
                                for (keyword, set) in changes {
                                    update.keyword(keyword.as_str(), set);
                                }
                            }
                        }
                    }
                }
            }

            // collect mailbox patches
            {
                let set_request = request.set_mailbox();

                for patch in patches.mailbox_patches {
                    let set = set_request.update(&patch.id);

                    if let Some(new_name) = patch.new_name {
                        set.name(new_name);
                    }

                    if let Some(new_parent_id) = patch.new_parent_id {
                        set.parent_id(new_parent_id);
                    }

                    if let Some(new_role) = patch.new_role {
                        set.role(new_role);
                    }

                    if let Some(new_sort_order) = patch.new_sort_order {
                        set.sort_order(new_sort_order);
                    }
                }
            }

            request.send().await?
        };

        // TODO: Error handling
        let mut mailbox_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_set_mailbox()
            .unwrap();

        mailbox_response.unwrap_update_errors().unwrap();

        let mailbox_result = remote::SetResult {
            value: Ok(mailbox_response
                .take_updated_ids()
                .map(|ids| ids.into_iter().map(MailboxId::from).collect())
                .unwrap_or_default()),
            old_state: mailbox_response.old_state().map(GetState::from),
            new_state: mailbox_response.take_new_state().into(),
        };

        let mut mail_response = response
            .pop_method_response()
            .unwrap()
            .unwrap_set_email()
            .unwrap();

        mail_response.unwrap_update_errors().unwrap();

        let mail_result = remote::SetResult {
            value: Ok(mail_response
                .take_updated_ids()
                .map(|ids| ids.into_iter().map(MailId::from).collect())
                .unwrap_or_default()),
            old_state: mail_response.old_state().map(GetState::from),
            new_state: mail_response.take_new_state().into(),
        };

        Ok((mailbox_result, mail_result))
    }
}
