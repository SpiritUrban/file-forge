use crate::job::ActiveJob;
use crate::models::JobProgress;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderSelection {
    pub input_path: String,
    pub output_path: String,
}

fn get_output_path(input_path_str: &str) -> String {
    let path = std::path::Path::new(input_path_str);
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        if let Some(parent) = path.parent() {
            // Keep correct path format for root drives or standard folders
            let parent_path = parent.to_string_lossy();
            if parent_path.is_empty() || parent_path == "\\" || parent_path == "/" {
                return format!("{}{}.optimized", parent_path, file_name);
            } else {
                return parent
                    .join(format!("{}.optimized", file_name))
                    .to_string_lossy()
                    .into_owned();
            }
        }
    }
    format!("{}.optimized", input_path_str)
}

#[tauri::command]
pub async fn select_folder() -> Result<Option<FolderSelection>, String> {
    let res = tauri::async_runtime::spawn_blocking(|| rfd::FileDialog::new().pick_folder())
        .await
        .map_err(|e| e.to_string())?;

    if let Some(path_buf) = res {
        let input_path = path_buf.to_string_lossy().into_owned();
        let output_path = get_output_path(&input_path);
        Ok(Some(FolderSelection {
            input_path,
            output_path,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn start_optimization(
    input_path: String,
    output_path: String,
    state: tauri::State<'_, Arc<ActiveJob>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut running = state.running.lock().unwrap();
    if *running {
        return Err("Job is already running".to_string());
    }
    *running = true;

    let active_job = state.inner().clone();
    let input = std::path::PathBuf::from(input_path);
    let output = std::path::PathBuf::from(output_path);

    std::thread::spawn(move || {
        crate::job::run_optimization_job(app, input, output, active_job);
    });

    Ok(())
}

#[tauri::command]
pub fn get_job_progress(state: tauri::State<'_, Arc<ActiveJob>>) -> Result<JobProgress, String> {
    Ok(state.progress.lock().unwrap().clone())
}

#[cfg(target_os = "windows")]
fn open_path(path: &str) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(path.replace('/', "\\"))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "windows"))]
fn open_path(path: &str) -> Result<(), String> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(cmd)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    open_path(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_output_path() {
        assert_eq!(get_output_path("D:\\Photos"), "D:\\Photos.optimized");
        assert_eq!(
            get_output_path("C:\\User\\Images"),
            "C:\\User\\Images.optimized"
        );
    }
}
