use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::task::TaskState;

#[derive(Serialize, Deserialize)]
pub struct TaskMomento {
    pub hash: String,
    pub file_path: PathBuf,
    pub url: String,
    pub total_bytes: Option<u64>,
    pub state: TaskState,
}

impl TaskMomento {
    pub fn new(
        hash: String,
        file_path: PathBuf,
        url: String,
        total_bytes: Option<u64>,
        state: TaskState,
    ) -> Self {
        Self {
            hash,
            file_path,
            url,
            total_bytes,
            state,
        }
    }
}
