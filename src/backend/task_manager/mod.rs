use crate::backend::mailbox::types::{MailboxId, ParentMailboxId};
use std::{cell::RefCell, collections::VecDeque};
use tokio::task::JoinHandle;

#[derive(Debug, PartialEq)]
pub enum TaskId {
    QueryChildMailboxes(ParentMailboxId),
    QueryRootMails(MailboxId),
}

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

    pub async fn finish_next_task(&self) {
        let _done = {
            let mut tasks = self.tasks.borrow_mut();
            match tasks.front_mut() {
                Some(task) => (&mut task.inner).await,
                None => std::future::pending().await,
            }
        };
        self.tasks.borrow_mut().pop_front();
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
    pub fn spawn<F>(&self, id: TaskId, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.is_already_running(&id) {
            return;
        }

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
