use serde::Serialize;

use crate::tasks::{Task, TaskState};

#[derive(Debug, Clone, Serialize)]
pub struct TaskJson {
    state: TaskState,
    bytes_received: u64,
    total_bytes: Option<u64>,
    progress: Option<f64>,
    speed: Option<f64>,

    filename: String,
    url: String,
    hash: String,
}

impl TaskJson {
    pub async fn from_task(task: &Task) -> Self {
        TaskJson {
            state: task.state().await.clone(),
            bytes_received: task.bytes_received().await,
            total_bytes: task.total_bytes().await,
            progress: task.progress().await,
            speed: task.average_speed().await,

            filename: task.filename().to_string(),
            url: task.url().as_str().to_string(),
            hash: task.hash().to_string(),
        }
    }
}
