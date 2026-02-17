use serde::Serialize;

use crate::task::TaskState;

#[derive(Debug, Clone, Serialize)]
pub struct TaskJson {
    state: TaskState,
    bytes_received: u64,
    total_bytes: Option<u64>,
    progress: Option<f64>,
    filename: String,
    url: String,
    hash: String,
}

impl TaskJson {
    pub fn new(
        state: TaskState,
        bytes_received: u64,
        total_bytes: Option<u64>,
        progress: Option<f64>,
        filename: String,
        url: String,
        hash: String,
    ) -> Self {
        TaskJson {
            state,
            bytes_received,
            total_bytes,
            progress,
            filename,
            url,
            hash,
        }
    }
}
