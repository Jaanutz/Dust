use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Running,
    Paused,
    Cancelled,
    Completed,
}
