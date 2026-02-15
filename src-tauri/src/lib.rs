use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    task_manager::TaskManager,
    tasks::{Task, TaskJson},
};

pub mod error;
pub mod network;
pub mod task_manager;
pub mod tasks;

type TaskManagerState = Arc<Mutex<TaskManager>>;

#[tauri::command]
async fn spawn_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<String>, String> {
    let manager = task_manager.lock().await;
    for hash in &hashes {
        manager.spawn_task(hash).await.map_err(|e| e.to_string())?;
    }

    Ok(hashes)
}

#[tauri::command]
async fn pause_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<String>, String> {
    let manager = task_manager.lock().await;
    for hash in &hashes {
        manager.pause_task(hash).await.map_err(|e| e.to_string())?;
    }

    Ok(hashes)
}

#[tauri::command]
async fn restart_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<String>, String> {
    let mut manager = task_manager.lock().await;
    for hash in &hashes {
        manager
            .restart_task(hash)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(hashes)
}

#[tauri::command]
async fn remove_tasks(
    hashes: Vec<String>,
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<String>, String> {
    let mut manager = task_manager.lock().await;
    for hash in &hashes {
        manager.remove_task(hash).await.map_err(|e| e.to_string())?;
    }

    Ok(hashes)
}

#[tauri::command]
async fn get_tasks(
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<TaskJson>, String> {
    let manager = task_manager.lock().await;
    let tasks = manager.list_tasks().await;
    Ok(tasks)
}

#[tauri::command]
async fn add_task(
    filename: String,
    file_path: String,
    url: String,
    task_manager: tauri::State<'_, TaskManagerState>,
) -> Result<String, String> {
    let mut manager = task_manager.lock().await;
    let task = Task::new(&filename, &file_path, &url, manager.client())
        .await
        .map_err(|e| e.to_string())?;
    let path = task.file_path().to_string_lossy().to_string();
    manager.add_task(task);

    Ok(path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let task_manager = Arc::new(Mutex::new(TaskManager::new()));

    tauri::Builder::default()
        .manage(task_manager)
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
