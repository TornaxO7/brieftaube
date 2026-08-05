use crate::backend::{Backend, MailboxData, MailboxId};

impl Backend {
    // pub fn get_or_request_mailbox_data(&self, id: &MailboxId) -> Option<MailboxData> {
    //     let data = self.get_mailbox_data(id);

    //     if data.is_none() {
    //         todo!();
    //         // self.request_mailbox_datas(&[id.to_owned()]);
    //     }

    //     data
    // }

    pub fn get_mailbox_data(&self, id: &MailboxId) -> Option<MailboxData> {
        let store = self.store.lock().unwrap();
        store.mailbox.get_data(id).cloned()
    }

    // fn request_mailbox_datas(&self, ids: &[MailboxId]) {
    //     let client = self.client.clone();
    //     let store = self.store.clone();
    //     let ids = ids.to_owned();

    //     self.task_manager.spawn(TaskId::MailboxGet, async move {
    //         let mut request = client.build();

    //         request_mailbox_datas(&mut request, &ids);

    //         let response = match request.send_get_mailbox().await {
    //             Ok(r) => r,
    //             Err(err) => {
    //                 error!("Couldn't send `Mailbox/get` request:\n{err}");
    //                 return;
    //             }
    //         };
    //     })
    // }
}

// fn request_mailbox_datas<'a>(request: &mut Request<'a>, ids: &[MailboxId]) {
//     request
//         .get_mailbox()
//         .properties(MailboxData::PROPERTIES)
//         .ids(Some(ids.iter().map(|id| &id.0)));
// }

// fn handle_mailbox_datas(store: &mut Store, response: MailboxGetResponse) {
//     store.set_root_mails(parent, root_mails);
// }
