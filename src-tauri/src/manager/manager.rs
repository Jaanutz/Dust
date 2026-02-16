use std::collections::HashMap;

use futures::future::join_all;
use reqwest::Client;
use tokio::task::JoinHandle;

use crate::{
    error::Error,
    task::{Task, TaskJson, TaskMomento},
    worker::DownloadWorker,
};

pub struct TaskManager {
    client: Client,
    workers: HashMap<String, DownloadWorker>,
}

impl TaskManager {
    pub async fn new() -> Self {
        let client = Client::builder()
            .user_agent(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            )
            .build()
            .unwrap_or(Client::new());

        let workers = Self::restore().await.unwrap_or_else(|_| HashMap::new());

        Self { client, workers }
    }

    pub fn get_worker(&self, hash: &str) -> Option<DownloadWorker> {
        self.workers.get(hash).cloned()
    }

    pub fn add_task(&mut self, task: Task) -> String {
        let hash = task.hash().to_string();
        let worker = DownloadWorker::new(task);
        self.workers.insert(hash.clone(), worker);
        hash
    }

    pub async fn remove_task(&mut self, hash: &str) -> Result<(), Error> {
        let worker = self.get_worker(hash);
        if let Some(worker) = worker {
            worker.abort().await;
            self.workers.remove(hash);
        } else {
            return Err(Error::TaskNotFound(hash.to_string()));
        }
        Ok(())
    }

    pub async fn pause_task(&self, hash: &str) -> Result<(), Error> {
        let worker = self.get_worker(hash);
        if let Some(worker) = worker {
            worker.pause().await?;
        } else {
            return Err(Error::TaskNotFound(hash.to_string()));
        }
        Ok(())
    }

    pub async fn spawn_task(&self, hash: &str) -> Result<Option<JoinHandle<()>>, Error> {
        let worker = self.get_worker(hash);
        if let Some(mut worker) = worker {
            return worker.spawn(self.client.clone()).await;
        }
        return Err(Error::TaskNotFound(hash.to_string()));
    }

    pub async fn restart_task(&self, hash: &str) -> Result<JoinHandle<()>, Error> {
        let worker = self.get_worker(hash);
        if let Some(mut worker) = worker {
            worker.abort().await;
            return Ok(worker.spawn(self.client.clone()).await?.unwrap());
        }
        return Err(Error::TaskNotFound(hash.to_string()));
    }

    pub async fn list_tasks(&self) -> Vec<TaskJson> {
        let futures = self
            .workers
            .iter()
            .map(|(_, worker)| async { worker.task().lock().await.to_json() });
        join_all(futures).await
    }
}

impl TaskManager {
    const TASKS_FILE: &str = "tasks.json";

    pub async fn save(&self) -> Result<(), Error> {
        let futures = self.workers.values().map(|worker| {
            let task = worker.task();
            async move { task.lock().await.snapshot() }
        });

        let snapshots = join_all(futures).await;
        let json = serde_json::to_string_pretty(&snapshots)?;
        std::fs::write(Self::TASKS_FILE, json)?;
        Ok(())
    }

    async fn restore() -> Result<HashMap<String, DownloadWorker>, Error> {
        let json = tokio::fs::read_to_string(Self::TASKS_FILE).await?;
        let snapshots: Vec<TaskMomento> = serde_json::from_str(&json)?;

        let mut workers = HashMap::new();
        for snapshot in snapshots {
            let hash = snapshot.hash.clone();
            let task = Task::restore(snapshot).await?;
            workers.insert(hash, DownloadWorker::new(task));
        }

        Ok(workers)
    }
}

impl TaskManager {
    pub fn client(&self) -> &Client {
        &self.client
    }
}
