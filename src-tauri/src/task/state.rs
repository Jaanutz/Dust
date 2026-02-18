use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskState {
    Running,
    Paused,
    Cancelled,
    Completed,
}
