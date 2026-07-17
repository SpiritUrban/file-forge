use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Idle,
    Scanning,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub status: JobStatus,
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
    pub optimized_files: usize,
    pub copied_files: usize,
    pub original_kept_files: usize,
    pub failed_files: usize,
    pub original_bytes: u64,
    pub output_bytes: u64,
}

impl Default for JobProgress {
    fn default() -> Self {
        Self {
            status: JobStatus::Idle,
            total_files: 0,
            processed_files: 0,
            current_file: None,
            optimized_files: 0,
            copied_files: 0,
            original_kept_files: 0,
            failed_files: 0,
            original_bytes: 0,
            output_bytes: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobOptions {
    pub convert_png_to_webp: bool,
    pub optimize_svg: bool,
    pub optimize_webp: bool,
}
