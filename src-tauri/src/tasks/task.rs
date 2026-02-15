use std::{path::PathBuf, sync::Arc};

use reqwest::{Client, StatusCode};
use sha1::{Digest, Sha1};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    select,
    sync::Mutex,
};
use tokio_util::{sync::CancellationToken};
use url::Url;

use crate::{
    error::Error,
    network::{fetch_total_bytes, get_best_extension},
    tasks::{TaskPersisted, TaskState},
};

#[derive(Clone)]
pub struct Task {
    state: Arc<Mutex<TaskState>>,
    bytes_received: Arc<Mutex<u64>>,
    total_bytes: Arc<Mutex<Option<u64>>>,

    cancellation_token: Arc<Mutex<CancellationToken>>,

    file_path: PathBuf,
    url: Url,
    hash: String,
}

impl Task {
    pub async fn new(
        filename: &str,
        file_path: &str,
        url: &str,
        client: &Client,
    ) -> Result<Self, Error> {
        let url = Url::parse(url)?;

        let full_path: PathBuf =
            Task::construct_full_file_path(client, filename, file_path, &url).await;
        Task::validate_full_file_path(&full_path)?;

        let full_path_str = full_path.to_str().ok_or(Error::IO(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid file path",
        )))?;

        let hash = Task::generate_file_path_hash(full_path_str);

        let mut task = Task {
            state: Arc::new(Mutex::new(TaskState::Paused)),
            bytes_received: Arc::new(Mutex::new(0)),
            total_bytes: Arc::new(Mutex::new(None)),
            cancellation_token: Arc::new(Mutex::new(CancellationToken::new())),
            file_path: full_path,
            url,
            hash,
        };
        task.initialize_metadata(client).await?;
        Ok(task)
    }

    async fn construct_full_file_path(
        client: &Client,
        filename: &str,
        file_path: &str,
        url: &Url,
    ) -> PathBuf {
        let mut full_path: PathBuf = PathBuf::from(file_path).join(filename);
        if let Some(ext) = get_best_extension(client, url).await {
            full_path.set_extension(ext);
        }
        full_path
    }

    fn validate_full_file_path(full_path: &PathBuf) -> Result<(), Error> {
        if full_path.exists() {
            return Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File already exists",
            )));
        }

        if full_path.parent().is_none() {
            return Err(Error::IO(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Directory does not exist",
            )));
        }

        Ok(())
    }

    fn generate_file_path_hash(full_path: &str) -> String {
        let part_file_hash_bytes = Sha1::digest(full_path);
        format!("{:x}", part_file_hash_bytes)
    }
}

impl Task {
    pub async fn from_persisted(persisted: TaskPersisted, client: &Client) -> Result<Self, Error> {
        let state = persisted.state;
        let url = Url::parse(&persisted.url)?;
        let file_path = PathBuf::from(persisted.file_path);
        let hash = persisted.hash;
        let total_bytes = persisted.total_bytes;
        let mut bytes_received = 0;
        if !matches!(state, TaskState::Completed) {
            Task::validate_full_file_path(&file_path)?;
        } else {
            bytes_received = total_bytes.unwrap_or(0);
        }

        let mut task = Task {
            state: Arc::new(Mutex::new(state)),
            bytes_received: Arc::new(Mutex::new(bytes_received)),
            total_bytes: Arc::new(Mutex::new(total_bytes)),
            cancellation_token: Arc::new(Mutex::new(CancellationToken::new())),
            file_path,
            url,
            hash,
        };
        task.initialize_metadata(client).await?;
        Ok(task)
    }
}

impl Task {
    async fn lazy_init_option<T, F>(opt: &Mutex<Option<T>>, f: F)
    where
        F: std::future::Future<Output = Option<T>>,
    {
        let mut lock = opt.lock().await;
        if lock.is_none() {
            if let Some(value) = f.await {
                *lock = Some(value);
            }
        }
    }

    pub async fn initialize_metadata(&mut self, client: &Client) -> Result<(), Error> {
        Task::lazy_init_option(&mut self.total_bytes, fetch_total_bytes(client, &self.url)).await;

        let part_file_path = self.part_file_path();
        if part_file_path.exists() {
            *self.bytes_received.lock().await = part_file_path.metadata()?.len();
        }

        Ok(())
    }
}

impl Task {
    async fn download(&mut self, client: &Client) -> Result<(), Error> {
        let url = self.url().as_str();
        let received_bytes = self.bytes_received().await;

        let request = client
            .get(url)
            .header("Range", format!("bytes={}-", received_bytes));
        let mut response = request.send().await?;
        response.error_for_status_ref()?;

        let status = response.status();
        if status != StatusCode::PARTIAL_CONTENT {
            response = client.get(url).send().await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.part_file_path())
            .await?;
        let cancellation_token = self.cancellation_token.lock().await.clone();

        loop {
            let chunk = select! {
                _ = cancellation_token.cancelled() => {
                    return Ok(());
                },
                chunk = response.chunk() => chunk,
            }?;

            let bytes = match chunk {
                Some(bytes) => bytes,
                None => break,
            };

            file.write_all(&bytes).await?;
            *self.bytes_received.lock().await += bytes.len() as u64;
        }

        file.flush().await?;
        file.sync_all().await?;

        fs::rename(self.part_file_path(), self.file_path()).await?;
        *self.state.lock().await = TaskState::Completed;

        Ok(())
    }
}

impl Task {
    pub async fn resume(&mut self, client: &Client) -> Result<(), Error> {
        let mut state_guard = self.state.lock().await;
        if matches!(*state_guard, TaskState::Completed | TaskState::Running) {
            return Ok(());
        }

        *state_guard = TaskState::Running;
        drop(state_guard);

        *self.cancellation_token.lock().await = CancellationToken::new();

        self.download(client).await
    }

    pub async fn pause(&mut self) {
        let mut state_guard = self.state.lock().await;
        if !matches!(*state_guard, TaskState::Running) {
            return;
        }

        self.cancel_token().await;
        *state_guard = TaskState::Paused;
    }

    pub async fn remove(&mut self) {
        self.cancel_token().await;
        *self.state.lock().await = TaskState::Cancelled;

        let _ = fs::remove_file(self.part_file_path()).await;
        let _ = fs::remove_file(self.file_path()).await;
        *self.bytes_received.lock().await = 0;
    }

    async fn cancel_token(&self) {
        self.cancellation_token.lock().await.cancel()
    }
}

impl Task {
    pub async fn state(&self) -> TaskState {
        self.state.lock().await.clone()
    }

    pub async fn bytes_received(&self) -> u64 {
        *self.bytes_received.lock().await
    }

    pub async fn total_bytes(&self) -> Option<u64> {
        self.total_bytes.lock().await.clone()
    }

    pub async fn progress(&self) -> Option<f64> {
        if let Some(total) = self.total_bytes().await {
            if total > 0 {
                return Some(self.bytes_received().await as f64 / total as f64);
            }
        }
        None
    }

    pub fn filename(&self) -> &str {
        self.file_path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn part_file_path(&self) -> PathBuf {
        let part_filename = format!("{}-{}", self.hash(), self.filename());
        PathBuf::from("./downloads/")
            .join(part_filename)
            .with_extension("part")
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }
}
