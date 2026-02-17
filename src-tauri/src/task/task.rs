use std::path::PathBuf;
use std::time::Instant;
use std::collections::VecDeque;

use reqwest::Client;
use sha1::{Digest, Sha1};
use tokio::fs::{self};
use url::Url;

use crate::{
    error::Error,
    network::{fetch_total_bytes, get_best_extension},
    task::{TaskJson, TaskMomento, TaskState},
};

pub struct Task {
    hash: String,
    file_path: PathBuf,
    url: Url,

    bytes_received: u64,
    history_bytes_received: Arc<Mutex<VecDeque<(Instant, u64)>>>,
    total_bytes: Option<u64>,
    state: TaskState,
}

impl Task {
    pub async fn new(
        filename: &str,
        file_path: &str,
        url: &str,
        client: &Client,
    ) -> Result<Self, Error> {
        let url = Url::parse(url)?;

        let mut file_path_with_name: PathBuf = PathBuf::from(file_path).join(filename);
        if let Some(extension) = get_best_extension(client, &url).await {
            file_path_with_name.set_extension(extension);
        }

        let hash_bytes = Sha1::digest(file_path_with_name.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hash_bytes);

        let total_bytes = fetch_total_bytes(client, &url).await;

        Ok(Task {
            state: TaskState::Paused,
            bytes_received: 0,
            history_bytes_received: Arc::new(Mutex::new(VecDeque::new())),
            total_bytes,
            file_path: file_path_with_name,
            url,
            hash,
        })
    }
}

impl Task {
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received
    }

    pub fn filename(&self) -> String {
        self.file_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn part_file_path(&self) -> PathBuf {
        let part_filename = format!("{}.part", self.hash);
        let parent = self
            .file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        parent.join(part_filename)
    }

    pub fn progress(&self) -> Option<f64> {
        if let Some(total) = self.total_bytes {
            if total > 0 {
                return Some(self.bytes_received as f64 / total as f64);
            }
        }
        None
    }

    pub fn state(&self) -> &TaskState {
        &self.state
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

impl Task {
    pub fn add_bytes_received(&mut self, bytes: u64) {
        self.bytes_received += bytes;
    }

    pub async fn finalize(&mut self) -> Result<(), Error> {
        fs::rename(self.part_file_path(), self.file_path.clone()).await?;
        self.state = TaskState::Completed;
        Ok(())
    }

    pub fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    pub fn reset_received_bytes(&mut self) {
        self.bytes_received = 0;
    }

    pub fn add_history_bytes_received(&mut self, bytes: u64) {
        self.history_bytes_received.push_back((Instant::now(), bytes));
        if (self.history_bytes_received.len() >= 15) {
            self.history_bytes_received.pop_front();
        }
    }

    pub async fn average_speed(&self) -> Option<f64>{
        let history = self.history_bytes_received.lock().await;
        if let (Some((start_time, start_bytes)), Some((end_time, end_bytes))) =
            (history.front(), history.back()) {
            let duration = end_time.duration_since(*start_time).as_secs_f64();
            if (duration > 0.1) {
                return Some((end_bytes - start_bytes) as f64 / duration as f64);
            }
        }
        None
    }
}

impl Task {
    pub fn snapshot(&self) -> TaskMomento {
        let original_state = self.state.clone();
        let state = if matches!(self.state, TaskState::Running) {
            TaskState::Paused
        } else {
            original_state
        };

        TaskMomento::new(
            self.hash.clone(),
            self.file_path.clone(),
            self.url.to_string(),
            self.total_bytes,
            state,
        )
    }

    pub async fn restore(snapshot: TaskMomento) -> Result<Self, Error> {
        let url = Url::parse(&snapshot.url)?;

        let mut task = Task {
            hash: snapshot.hash,
            file_path: snapshot.file_path,
            url,
            bytes_received: 0,
            total_bytes: snapshot.total_bytes,
            history_bytes_received: Arc::new(Mutex::new(VecDeque::new())),
            state: snapshot.state,
        };

        task.bytes_received = if matches!(task.state, TaskState::Completed) {
            task.total_bytes.unwrap_or(0)
        } else {
            let part_path = task.part_file_path();
            fs::metadata(&part_path).await.map(|meta| meta.len())?
        };

        Ok(task)
    }
}

impl Task {
    pub fn to_json(&self) -> TaskJson {
        TaskJson::new(
            self.state.clone(),
            self.bytes_received,
            self.total_bytes,
            self.progress(),
            self.filename(),
            self.url.to_string(),
            self.hash.clone(),
        )
    }
}
