use async_trait::async_trait;
use reqwest::Client;

use crate::{error::Error, task::TaskState, worker::DownloadWorker};

pub struct CommandContext<'a> {
    pub worker: &'a mut DownloadWorker,
    pub client: &'a Client,
}

#[async_trait]
pub trait Command: Copy + Clone {
    fn can_execute(&self, state: TaskState) -> bool;

    async fn execute(&self, context: &CommandContext) -> Result<(), Error>;
}

#[derive(Clone, Copy)]
pub struct SpawnCommand;

#[async_trait]
impl Command for SpawnCommand {
    fn can_execute(&self, state: TaskState) -> bool {
        matches!(state, TaskState::Paused | TaskState::Cancelled)
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Error> {
        context
            .worker
            .with_task_mut(|task| {
                task.set_state(TaskState::Running);
                task.clear_history();
            })
            .await;

        context.worker.reset_cancellation_token().await;

        let worker = context.worker.clone();
        let client = context.client.clone();

        tokio::spawn(async move {
            worker.run(&client).await.unwrap();
        });
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct PauseCommand;

#[async_trait]
impl Command for PauseCommand {
    fn can_execute(&self, state: TaskState) -> bool {
        !matches!(state, TaskState::Running)
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Error> {
        context.worker.cancel().await;
        context
            .worker
            .with_task_mut(|task| task.set_state(TaskState::Paused))
            .await;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct AbortCommand;

#[async_trait]
impl Command for AbortCommand {
    fn can_execute(&self, _: TaskState) -> bool {
        true
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Error> {
        context.worker.cancel().await;

        let (part_path, file_path) = context
            .worker
            .with_task_mut(|task| {
                task.set_state(TaskState::Cancelled);
                task.reset_received_bytes();
                (task.part_file_path(), task.file_path().clone())
            })
            .await;

        let _ = tokio::fs::remove_file(part_path).await;
        let _ = tokio::fs::remove_file(file_path).await;

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RestartCommand;

#[async_trait::async_trait]
impl Command for RestartCommand {
    fn can_execute(&self, _: TaskState) -> bool {
        true
    }

    async fn execute(&self, context: &CommandContext) -> Result<(), Error> {
        let abort = AbortCommand;
        let spawn = SpawnCommand;

        abort.execute(context).await?;
        spawn.execute(context).await?;
        Ok(())
    }
}
