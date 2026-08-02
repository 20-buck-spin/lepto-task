use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::task_models::Task;

#[derive(Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub task: Task,
    pub next_exec_dt: DateTime<Utc>,
}

impl PartialEq for QueuedTask {
    fn eq(&self, other: &Self) -> bool {
        self.next_exec_dt == other.next_exec_dt
    }
}
impl Eq for QueuedTask {}

impl PartialOrd for QueuedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.next_exec_dt.cmp(&other.next_exec_dt).reverse())
    }
}
impl Ord for QueuedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next_exec_dt.cmp(&other.next_exec_dt).reverse()
    }
}
