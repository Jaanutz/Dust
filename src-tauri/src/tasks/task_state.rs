use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum TaskState {
    Running,
    Paused,
    Cancelled,
    Completed,
}
