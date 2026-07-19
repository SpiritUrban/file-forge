use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Finds `ffmpeg` in PATH and returns its full path.
pub fn find_ffmpeg() -> Option<std::path::PathBuf> {
    let possible_local_paths = [
        "ffmpeg-8.1.2-essentials_build/bin/ffmpeg.exe",
        "../ffmpeg-8.1.2-essentials_build/bin/ffmpeg.exe",
    ];

    for path in possible_local_paths {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            if let Ok(canonical) = std::fs::canonicalize(&p) {
                return Some(canonical);
            }
            return Some(p);
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let bundled = parent.join("ffmpeg-8.1.2-essentials_build/bin/ffmpeg.exe");
            if bundled.exists() {
                return Some(bundled);
            }
        }
    }

    which::which("ffmpeg").ok()
}

/// Returns true if ffmpeg is available on this system.
pub fn ffmpeg_available() -> bool {
    find_ffmpeg().is_some()
}

// ---------------------------------------------------------------------------
// Internal helper — polls a running child process and respects cancellation.
// ---------------------------------------------------------------------------
fn wait_for_ffmpeg(
    mut child: std::process::Child,
    output_path: &Path,
    active_job: &Arc<crate::ActiveJob>,
) -> Result<(), String> {
    loop {
        if active_job.cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
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
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                return Err(format!("Помилка очікування FFmpeg: {e}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared FFmpeg spawn helper.
// ---------------------------------------------------------------------------
fn spawn_ffmpeg<'a>(
    ffmpeg: &std::path::Path,
    args: impl IntoIterator<Item = &'a str>,
) -> Result<std::process::Child, String> {
    std::process::Command::new(ffmpeg)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Не вдалося запустити FFmpeg: {e}"))
}

// ---------------------------------------------------------------------------
// Public converters
// ---------------------------------------------------------------------------

/// Converts VOB/AVI → MP4 (libx264 + AAC, streaming-ready).
pub fn convert_video(
    input_path: &Path,
    output_path: &Path,
    crf: u8,
    use_h265: bool,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let crf_str = crf.to_string();
    let input = input_path.to_str().unwrap_or_default();
    let output = output_path.to_str().unwrap_or_default();

    let child = spawn_ffmpeg(
        &ffmpeg,
        [
            "-y", "-i", input,
            "-c:v", if use_h265 { "libx265" } else { "libx264" },
            "-crf", &crf_str,
            "-preset", "medium",
            "-c:a", "aac",
            "-b:a", "128k",
            "-movflags", "+faststart",
            output,
        ],
    )?;

    wait_for_ffmpeg(child, output_path, &active_job)
}

/// Re-encodes an existing MP4 to a smaller size using YouTube-like settings:
/// higher CRF (28) + slow preset + AAC 128k + faststart.
pub fn optimize_mp4(
    input_path: &Path,
    output_path: &Path,
    crf: u8,
    use_h265: bool,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let crf_str = crf.to_string();
    let input = input_path.to_str().unwrap_or_default();
    let output = output_path.to_str().unwrap_or_default();

    let child = spawn_ffmpeg(
        &ffmpeg,
        [
            "-y", "-i", input,
            "-c:v", if use_h265 { "libx265" } else { "libx264" },
            "-crf", &crf_str,
            "-preset", "slow",   // slower = better compression ratio
            "-c:a", "aac",
            "-b:a", "128k",
            "-movflags", "+faststart",
            output,
        ],
    )?;

    wait_for_ffmpeg(child, output_path, &active_job)
}

/// Converts WAV → MP3 using libmp3lame at the given bitrate (kbps).
pub fn convert_wav_to_mp3(
    input_path: &Path,
    output_path: &Path,
    bitrate_kbps: u32,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let bitrate_str = format!("{}k", bitrate_kbps);
    let input = input_path.to_str().unwrap_or_default();
    let output = output_path.to_str().unwrap_or_default();

    let child = spawn_ffmpeg(
        &ffmpeg,
        [
            "-y", "-i", input,
            "-c:a", "libmp3lame",
            "-b:a", &bitrate_str,
            "-id3v2_version", "3",   // widely-compatible ID3 tags
            output,
        ],
    )?;

    wait_for_ffmpeg(child, output_path, &active_job)
}

/// Extracts audio from a video file and saves it as MP3.
pub fn extract_audio_to_mp3(
    input_path: &Path,
    output_path: &Path,
    bitrate_kbps: u32,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let bitrate_str = format!("{}k", bitrate_kbps);
    let input = input_path.to_str().unwrap_or_default();
    let output = output_path.to_str().unwrap_or_default();

    let child = spawn_ffmpeg(
        &ffmpeg,
        [
            "-y", "-i", input,
            "-vn", // skip video
            "-c:a", "libmp3lame",
            "-b:a", &bitrate_str,
            "-id3v2_version", "3",
            output,
        ],
    )?;

    wait_for_ffmpeg(child, output_path, &active_job)
}

/// Converts an animated GIF to a silent MP4.
pub fn convert_gif_to_mp4(
    input_path: &Path,
    output_path: &Path,
    active_job: Arc<crate::ActiveJob>,
) -> Result<(), String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| {
        "FFmpeg не знайдено у системі. Встановіть FFmpeg і перезапустіть застосунок.".to_string()
    })?;

    let input = input_path.to_str().unwrap_or_default();
    let output = output_path.to_str().unwrap_or_default();

    let child = spawn_ffmpeg(
        &ffmpeg,
        [
            "-y", "-i", input,
            "-c:v", "libx264",
            "-crf", "23",
            "-preset", "medium",
            "-pix_fmt", "yuv420p", // essential for wide compatibility of GIF->MP4
            "-an", // no audio in GIFs
            "-movflags", "+faststart",
            output,
        ],
    )?;

    wait_for_ffmpeg(child, output_path, &active_job)
}
