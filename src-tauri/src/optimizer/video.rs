use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Finds `ffmpeg` in PATH and returns its full path.
pub fn find_ffmpeg() -> Option<std::path::PathBuf> {
    which::which("ffmpeg").ok()
}

/// Returns true if ffmpeg is available on this system.
pub fn ffmpeg_available() -> bool {
    find_ffmpeg().is_some()
}

/// Converts a video file (VOB/AVI) to MP4 using libx264 + AAC.
///
/// - `crf`: Constant Rate Factor (0–51). Lower = better quality. Default: 23.
/// - `active_job`: shared job state — `.cancelled` flag is polled every 200ms.
///   If set, the process is killed and the incomplete output file is removed.
pub fn convert_video(
    input_path: &Path,
    output_path: &Path,
    crf: u8,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let crf_str = crf.to_string();

    let mut child = std::process::Command::new(&ffmpeg)
        .args([
            "-y", // overwrite output without asking
            "-i",
            input_path.to_str().unwrap_or_default(),
            "-c:v",
            "libx264",
            "-crf",
            &crf_str,
            "-preset",
            "medium",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-movflags",
            "+faststart", // streaming-friendly MP4
            output_path.to_str().unwrap_or_default(),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити FFmpeg: {e}"))?;

    // Poll for completion, checking cancellation flag every 200ms
    loop {
        if active_job.cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
            // Remove incomplete output file
            let _ = std::fs::remove_file(output_path);
            return Err("Скасовано".to_string());
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                } else {
                    return Err(format!(
                        "FFmpeg завершився з помилкою (код: {:?})",
                        status.code()
                    ));
                }
            }
            Ok(None) => {
                // Still running, wait a bit
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                return Err(format!("Помилка очікування FFmpeg: {e}"));
            }
        }
    }
}
