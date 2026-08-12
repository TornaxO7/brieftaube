use std::cell::RefCell;

use tokio::task::JoinSet;

pub struct TaskManager {
    tasks: RefCell<JoinSet<()>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RefCell::new(JoinSet::new()),
        }
    }

    pub async fn finish_next_task(&self) {
        if self.tasks.borrow().is_empty() {
            std::future::pending::<()>().await;
        }

        self.tasks.borrow_mut().join_next().await;
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.borrow().is_empty()
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.borrow_mut().spawn(future);
    }
}
