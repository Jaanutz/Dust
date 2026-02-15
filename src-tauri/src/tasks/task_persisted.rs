use serde::{Deserialize, Serialize};

use crate::tasks::{Task, TaskState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPersisted {
    pub state: TaskState,
    pub total_bytes: Option<u64>,

    pub file_path: String,
    pub url: String,
    pub hash: String,
}

impl TaskPersisted {
    pub async fn from_task(task: &Task) -> Self {
        let task_state = task.state().await;
        let state;
        if matches!(task_state, TaskState::Running) {
            state = TaskState::Paused;
        } else {
            state = task_state.clone();
        }

        Self {
            state,
            total_bytes: task.total_bytes().await,
            file_path: task.file_path().to_str().unwrap_or_default().into(),
            url: task.url().as_str().into(),
            hash: task.hash().into(),
        }
    }
}
