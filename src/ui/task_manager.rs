use std::cell::RefCell;
use tokio::task::JoinSet;

// TODO:
// - Create `Task` struct which contains error message (if there's any)
// - Store all tasks

pub type TaskResult = Vec<crate::ui::Message>;

pub struct TaskManager {
    tasks: RefCell<JoinSet<TaskResult>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RefCell::new(JoinSet::new()),
        }
    }

    pub async fn finish_next_task(&self) -> Vec<super::Message> {
        if self.tasks.borrow().is_empty() {
            std::future::pending::<()>().await;
        }

        self.tasks.borrow_mut().join_next().await.unwrap().unwrap()
    }

    pub fn has_tasks_running(&self) -> bool {
        !self.tasks.borrow().is_empty()
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = Vec<super::Message>> + Send + 'static,
    {
        self.tasks.borrow_mut().spawn(future);
    }
}
