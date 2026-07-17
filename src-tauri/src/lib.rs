pub mod commands;
pub mod job;
pub mod models;
pub mod optimizer;
pub mod scanner;

use crate::job::ActiveJob;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let active_job = Arc::new(ActiveJob::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(active_job)
        .invoke_handler(tauri::generate_handler![
            commands::select_folder,
            commands::start_optimization,
            commands::get_job_progress,
            commands::open_folder,
            commands::cancel_optimization
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
