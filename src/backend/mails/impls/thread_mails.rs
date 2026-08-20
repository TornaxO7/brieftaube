use crate::backend::{Backend, LoadingRole, MailData, MailId, ThreadId, types::Loadable};
use tokio::sync::watch;

// impl Backend {
//     pub async fn get_thread_mail_ids(
//         &self,
//         id: &ThreadId,
//     ) -> Result<Vec<MailId>, jmap_client::Error> {
//         let role = {
//             let mut store = self.store.lock().unwrap();
//             let ids = store.threads.get_mails_mut(id);

//             match ids {
//                 Loadable::NotRequested => {
//                     let (tx, rx) = watch::channel(());
//                     *ids = Loadable::Requested { notifier: rx };
//                     FetchRole::Request(tx)
//                 }
//                 Loadable::Requested { notifier } => FetchRole::Wait(notifier.clone()),
//                 Loadable::Loaded(ids) => return Ok(ids.clone()),
//             }
//         };

//         match role {
//             FetchRole::Wait(mut receiver) => {
//                 receiver.changed().await.unwrap();
//                 let mut store = self.store.lock().unwrap();
//                 let ids = store.threads.get_mails(id).loaded().unwrap();
//                 Ok(ids.clone())
//             }
//             FetchRole::Request(_sender) => {
//                 unreachable!("Should be requested from the logic. Otherwise: Fill this");
//             }
//         }
//     }

//     pub async fn get_thread_mails(
//         &self,
//         id: &ThreadId,
//     ) -> Result<Vec<MailData>, jmap_client::Error> {
//         let ids = self.get_thread_mail_ids(id).await?;
//         self.get_mails(&ids).await
//     }
// }
