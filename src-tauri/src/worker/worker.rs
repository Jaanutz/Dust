use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Instant;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, select, spawn, sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    task::{Task, TaskState},
};

#[derive(Clone)]
pub struct DownloadWorker {
    task: Arc<Mutex<Task>>,
    cancellation_token: Arc<Mutex<CancellationToken>>,
}

impl DownloadWorker {
    pub fn new(task: Task) -> Self {
        Self {
            task: Arc::new(Mutex::new(task)),
            cancellation_token: Arc::new(Mutex::new(CancellationToken::new())),
        }
    }
}

impl DownloadWorker {
    pub fn task(&self) -> Arc<Mutex<Task>> {
        self.task.clone()
    }
}

impl DownloadWorker {
    pub async fn run(&self, client: &Client) -> Result<(), Error> {
        let task_guard = self.task.lock().await;
        let start = task_guard.bytes_received();
        let url = task_guard.url().to_string();
        let part_file_path = task_guard.part_file_path();
        drop(task_guard);

        let mut request = client.get(&url);
        if start > 0 {
            request = request.header("Range", format!("bytes={}-", start));
        }

        let mut response = request.send().await?;
        let mut has_to_start_from_beginning = false;
        if response.status() != StatusCode::PARTIAL_CONTENT {
            response = client.get(&url).send().await?;
            has_to_start_from_beginning = true;
            self.task.lock().await.reset_received_bytes();
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(!has_to_start_from_beginning)
            .truncate(has_to_start_from_beginning)
            .open(&part_file_path)
            .await?;

        let cancellation_token = self.cancellation_token.clone();
        let mut last_history_push = Instant::now();
        let min_millis_since_push = 200;
        loop {
            let token_guard = cancellation_token.lock().await;
            let chunk = {
                select! {
                    _ = token_guard.cancelled() => {
                        return Ok(());
                    },
                    chunk = response.chunk() => chunk,
                }?
            };
            drop(token_guard);

            let bytes = match chunk {
                Some(bytes) => bytes,
                None => break,
            };

            file.write_all(&bytes).await?;

            self.task
                .lock()
                .await
                .add_bytes_received(bytes.len() as u64);

            let now = Instant::now();
            if now.duration_since(last_history_push).as_millis() > min_millis_since_push as u128 {
                self.task
                    .lock()
                    .await
                    .update_history_bytes_received(Instant::now());
                last_history_push = Instant::now();
            }
        }

        file.sync_all().await?;

        self.task.lock().await.finalize().await?;

        Ok(())
    }

    async fn cancel(&self) {
        self.cancellation_token.lock().await.cancel();
    }
}

impl DownloadWorker {
    pub async fn spawn(&mut self, client: Client) -> Result<Option<JoinHandle<()>>, Error> {
        let mut task_guard = self.task.lock().await;

        if matches!(
            *task_guard.state(),
            TaskState::Running | TaskState::Completed
        ) {
            return Ok(None);
        }

        task_guard.set_state(TaskState::Running);
        task_guard.clear_history();
        drop(task_guard);

        *self.cancellation_token.lock().await = CancellationToken::new();
        let worker = self.clone();

        Ok(Some(spawn(
            async move { worker.run(&client).await.unwrap() },
        )))
    }

    pub async fn pause(&self) -> Result<(), Error> {
        let mut task_guard = self.task.lock().await;

        if !matches!(*task_guard.state(), TaskState::Running) {
            return Ok(());
        }

        self.cancel().await;

        task_guard.set_state(TaskState::Paused);

        Ok(())
    }

    pub async fn abort(&self) {
        self.cancel().await;

        let mut task_guard = self.task.lock().await;

        let _ = tokio::fs::remove_file(task_guard.part_file_path()).await;
        let _ = tokio::fs::remove_file(task_guard.file_path()).await;

        task_guard.set_state(TaskState::Cancelled);
        task_guard.reset_received_bytes();
    }
}
