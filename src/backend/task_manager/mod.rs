use crate::backend::{
    mailbox::types::{MailboxId, ParentMailboxId},
    threads::types::ThreadId,
};
use std::{cell::RefCell, collections::VecDeque};
use tokio::task::JoinHandle;
use tracing::{debug, instrument};

#[derive(Debug, PartialEq, Hash)]
pub enum TaskId {
    QueryChildMailboxes(ParentMailboxId),
    QueryRootMails(MailboxId),
    GetThreadMails,
}

/// Add waiting-time for each task to avoid too many requests.
pub struct TaskManager {
    tasks: RefCell<VecDeque<Task>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RefCell::new(VecDeque::with_capacity(16)),
        }
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.borrow_mut().is_empty()
    }

    #[instrument(skip(self))]
    pub async fn finish_next_task(&self) {
        let _done = {
            let mut tasks = self.tasks.borrow_mut();
            match tasks.front_mut() {
                Some(task) => (&mut task.inner).await,
                None => std::future::pending().await,
            }
        };
        if let Some(finished_task) = self.tasks.borrow_mut().pop_front() {
            debug!("{:?} finished.", finished_task.id);
        }
    }
}

/// helper
impl TaskManager {
    fn is_already_running(&self, id: &TaskId) -> bool {
        self.tasks
            .borrow()
            .iter()
            .find(|task| &task.id == id)
            .is_some()
    }
}

/// starting tasks
impl TaskManager {
    #[instrument(skip(self, future))]
    pub fn spawn<F>(&self, id: TaskId, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.is_already_running(&id) {
            debug!("'{id:?}' is already running. Abort.");
            return;
        }
        debug!("Spawning '{id:?}'");

        let new_task = Task {
            id,
            inner: tokio::spawn(future),
        };

        self.tasks.borrow_mut().push_back(new_task);
    }
}

struct Task {
    id: TaskId,
    inner: JoinHandle<()>,
}
