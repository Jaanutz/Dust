use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Instant;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, select, sync::Mutex};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    task::Task,
    worker::{Command, CommandContext},
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
    pub async fn with_task<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Task) -> R,
    {
        let task = self.task.lock().await;
        f(&task)
    }

    pub(super) async fn with_task_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Task) -> R,
    {
        let mut task = self.task.lock().await;
        f(&mut task)
    }
}

impl DownloadWorker {
    pub(super) async fn run(&self, client: &Client) -> Result<(), Error> {
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

    pub(super) async fn cancel(&self) {
        self.cancellation_token.lock().await.cancel();
    }
}

impl DownloadWorker {
    pub async fn run_command<C: Command>(
        &mut self,
        command: C,
        client: &Client,
    ) -> Result<(), Error> {
        let mut context = CommandContext {
            worker: self,
            client,
        };

        if !command.can_execute(context.worker.task.lock().await.state()) {
            return Ok(());
        }

        command.execute(&mut context).await
    }

    pub async fn reset_cancellation_token(&self) {
        *self.cancellation_token.lock().await = CancellationToken::new();
    }
}
