use std::collections::HashMap;

use futures::future::join_all;
use reqwest::Client;
use tokio::{spawn, task::JoinHandle};

use crate::{
    error::Error,
    tasks::{Task, TaskJson, TaskPersisted},
};

pub struct TaskManager {
    tasks: HashMap<String, Task>,
    client: reqwest::Client,
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

        let tasks = TaskManager::load_tasks(&client)
            .await
            .unwrap_or_else(|_| HashMap::new());

        Self { tasks, client }
    }

    pub fn add_task(&mut self, task: Task) -> String {
        let hash = task.hash().to_string();
        self.tasks.insert(hash.clone(), task);
        hash
    }

    pub async fn remove_task(&mut self, hash: &str) -> Result<(), Error> {
        let mut task = self.get_task_clone(hash)?;
        task.remove().await;
        self.tasks.remove(hash);
        Ok(())
    }

    pub async fn pause_task(&self, hash: &str) -> Result<(), Error> {
        let mut task = self.get_task_clone(hash)?;
        task.pause().await;
        Ok(())
    }

    pub async fn spawn_task(&self, hash: &str) -> Result<JoinHandle<()>, Error> {
        let mut task = self.get_task_clone(hash)?;
        let client = self.client.clone();
        Ok(spawn(async move { task.resume(&client).await.unwrap() }))
    }

    pub async fn restart_task(&mut self, hash: &str) -> Result<JoinHandle<()>, Error> {
        let mut task = self.get_task_clone(hash)?;
        task.remove().await;
        let client = self.client.clone();
        Ok(spawn(async move { task.resume(&client).await.unwrap() }))
    }

    pub fn get_task_clone(&self, hash: &str) -> Result<Task, Error> {
        let task = self
            .tasks
            .get(hash)
            .ok_or_else(|| Error::TaskNotFound(hash.to_string()))?
            .clone();
        Ok(task)
    }

    pub async fn list_tasks(&self) -> Vec<TaskJson> {
        let futures = self
            .tasks
            .iter()
            .map(|(_, task)| async { TaskJson::from_task(task).await });
        join_all(futures).await
    }

    pub async fn list_received_bytes(&self) -> Vec<(String, u64)> {
        let futures = self.tasks.iter().map(|(hash, task)| {
            let hash = hash.clone();
            async {
                let bytes = task.bytes_received().await;
                (hash, bytes)
            }
        });
        join_all(futures).await
    }
}

impl TaskManager {
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl TaskManager {
    pub async fn save_tasks(&self) -> Result<(), Error> {
        let mut persisted = Vec::new();

        for task in self.tasks.values() {
            persisted.push(TaskPersisted::from_task(task).await);
        }

        let json = serde_json::to_string_pretty(&persisted)?;
        std::fs::write("tasks.json", json)?;
        Ok(())
    }

    pub async fn load_tasks(client: &Client) -> Result<HashMap<String, Task>, Error> {
        let json = tokio::fs::read_to_string("tasks.json").await?;
        let persisted: Vec<TaskPersisted> = serde_json::from_str(&json)?;

        let mut tasks = HashMap::new();

        for task in persisted {
            let hash = task.hash.clone();
            let task = Task::from_persisted(task, client).await?;
            tasks.insert(hash, task);
        }

        Ok(tasks)
    }
}
