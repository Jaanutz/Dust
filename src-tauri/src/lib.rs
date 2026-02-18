use std::sync::Arc;

use tauri::WindowEvent;
use tokio::sync::Mutex;

use crate::{
    manager::TaskManager,
    task::{Task, TaskJson},
    worker::command::{AbortCommand, PauseCommand, RestartCommand, SpawnCommand},
};

pub mod error;
pub mod manager;
pub mod network;
pub mod task;
pub mod worker;

type SharedTaskManager = Arc<Mutex<TaskManager>>;

#[tauri::command]
async fn spawn_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<Vec<String>, String> {
    let task_manager = task_manager.lock().await;
    task_manager.run_command_vec(&hashes, SpawnCommand).await?;

    Ok(hashes)
}

#[tauri::command]
async fn pause_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<Vec<String>, String> {
    let task_manager = task_manager.lock().await;
    task_manager.run_command_vec(&hashes, PauseCommand).await?;

    Ok(hashes)
}

#[tauri::command]
async fn restart_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<Vec<String>, String> {
    let task_manager = task_manager.lock().await;
    task_manager
        .run_command_vec(&hashes, RestartCommand)
        .await?;

    Ok(hashes)
}

#[tauri::command]
async fn remove_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<Vec<String>, String> {
    let mut task_manager = task_manager.lock().await;
    task_manager.run_command_vec(&hashes, AbortCommand).await?;

    for hash in hashes.iter() {
        task_manager.remove_task(hash).map_err(|e| e.to_string())?;
    }

    Ok(hashes)
}

#[tauri::command]
async fn get_tasks(
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<Vec<TaskJson>, String> {
    let task_manager = task_manager.lock().await;
    let tasks = task_manager.list_tasks().await;

    Ok(tasks)
}

#[tauri::command]
async fn add_task(
    filename: String,
    file_path: String,
    url: String,
    task_manager: tauri::State<'_, SharedTaskManager>,
) -> Result<String, String> {
    let mut task_manager = task_manager.lock().await;
    let task = Task::new(&filename, &file_path, &url, task_manager.client())
        .await
        .map_err(|e| e.to_string())?;
    let path = task.file_path().to_string_lossy().to_string();
    task_manager.add_task(task);

    Ok(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_manager = tauri::async_runtime::block_on(async {
        let manager = TaskManager::new().await;
        Arc::new(Mutex::new(manager))
    });

    tauri::Builder::default()
        .manage(task_manager.clone())
        .on_window_event(move |_window, event| {
            if let WindowEvent::CloseRequested { api: _, .. } = event {
                let storing_task_manager = task_manager.clone();
                tauri::async_runtime::block_on(async move {
                    storing_task_manager.lock().await.save().await.unwrap();
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            spawn_tasks,
            pause_tasks,
            remove_tasks,
            restart_tasks,
            get_tasks,
            add_task
        ])
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
