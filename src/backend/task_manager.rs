use tokio::task::JoinSet;

#[derive(Default)]
pub struct TaskManager {
    tasks: JoinSet<()>,
}

impl TaskManager {
    pub async fn task_finished(&self) {
        self.tasks.join_next().await;
    }
}
