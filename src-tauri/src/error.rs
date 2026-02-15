use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Parse URL error: {0}")]
    ParseURL(#[from] url::ParseError),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Serde JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
