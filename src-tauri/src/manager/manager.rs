use std::collections::HashMap;

use futures::future::join_all;
use reqwest::Client;

use crate::{
    error::Error,
    task::{Task, TaskJson, TaskMemento},
    worker::{Command, CommandContext, DownloadWorker},
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

    pub fn remove_task(&mut self, hash: &str) -> Result<(), Error> {
        if self.workers.remove(hash).is_some() {
            Ok(())
        } else {
            Err(Error::TaskNotFound(hash.to_string()))
        }
    }
}

impl TaskManager {
    pub async fn run_command(&self, hash: &str, command: impl Command) -> Result<(), Error> {
        let worker = self.get_worker(hash);
        if let Some(mut worker) = worker {
            let context = CommandContext {
                worker: &mut worker,
                client: &self.client,
            };
            command.execute(&context).await?;
        } else {
            return Err(Error::TaskNotFound(hash.to_string()));
        }
        Ok(())
    }

    pub async fn run_command_vec<C: Command>(
        &self,
        hashes: &[String],
        command: C,
    ) -> Result<(), String> {
        let futures = hashes.iter().map(|hash| {
            let command = command.clone();
            async move { self.run_command(hash, command).await }
        });

        let results = join_all(futures).await;
        results
            .into_iter()
            .try_for_each(|r| r)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn list_tasks(&self) -> Vec<TaskJson> {
        let futures = self.workers.iter().map(|(_, worker)| async {
            let json = worker.with_task(|task| task.to_json()).await;
            json
        });

        join_all(futures).await
    }
}

impl TaskManager {
    const TASKS_FILE: &str = "tasks.json";

    pub async fn save(&self) -> Result<(), Error> {
        let workers: Vec<_> = self.workers.values().cloned().collect();

        let futures = workers.into_iter().map(|worker| async move {
            let snapshot = worker.with_task(|task| task.snapshot()).await;
            snapshot
        });

        let snapshots = join_all(futures).await;

        let json = serde_json::to_string_pretty(&snapshots)?;
        std::fs::write(Self::TASKS_FILE, json)?;
        Ok(())
    }

    async fn restore() -> Result<HashMap<String, DownloadWorker>, Error> {
        let json = tokio::fs::read_to_string(Self::TASKS_FILE).await?;
        let snapshots: Vec<TaskMemento> = serde_json::from_str(&json)?;

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
