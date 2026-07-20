import re

def main():
    with open('src-tauri/src/optimizer/video.rs', 'r', encoding='utf-8') as f:
        content = f.read()

    # spawn_ffmpeg
    content = content.replace('.stderr(std::process::Stdio::null())', '.stderr(std::process::Stdio::piped())')
    
    # wait_for_ffmpeg signature
    content = content.replace(
        'fn wait_for_ffmpeg(\n    mut child: std::process::Child,\n    output_path: &Path,\n    active_job: &Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'fn wait_for_ffmpeg(\n    mut child: std::process::Child,\n    output_path: &Path,\n    active_job: &Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n    total_duration: Option<f64>,\n) -> Result<(), String> {\n    use std::io::{BufRead, BufReader};\n    use tauri::Emitter;\n\n    if let Some(stderr) = child.stderr.take() {\n        let active_job = active_job.clone();\n        let app = app.clone();\n        std::thread::spawn(move || {\n            let reader = BufReader::new(stderr);\n            for line in reader.lines() {\n                if active_job.cancelled.load(Ordering::Relaxed) {\n                    break;\n                }\n                let Ok(line) = line else { break };\n                if let Some(total_dur) = total_duration {\n                    if let Some(time_idx) = line.find("time=") {\n                        let time_str = &line[time_idx + 5..];\n                        let parts: Vec<&str> = time_str.split(\':\').collect();\n                        if parts.len() == 3 {\n                            let h = parts[0].parse::<f64>().unwrap_or(0.0);\n                            let m = parts[1].parse::<f64>().unwrap_or(0.0);\n                            let s_parts: Vec<&str> = parts[2].split_whitespace().collect();\n                            if !s_parts.is_empty() {\n                                let s = s_parts[0].parse::<f64>().unwrap_or(0.0);\n                                let current_dur = h * 3600.0 + m * 60.0 + s;\n                                let mut progress = (current_dur / total_dur) * 100.0;\n                                if progress > 100.0 { progress = 100.0; }\n                                if progress < 0.0 { progress = 0.0; }\n                                \n                                {\n                                    let mut prog_lock = active_job.progress.lock().unwrap();\n                                    prog_lock.current_file_progress = Some(progress as f32);\n                                }\n                                let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());\n                            }\n                        }\n                    }\n                }\n            }\n        });\n    }'
    )

    # All public converter signatures need app: &tauri::AppHandle
    
    # 1. extract_audio_to_mp3
    content = content.replace(
        'pub fn extract_audio_to_mp3(\n    input: &Path,\n    output: &Path,\n    bitrate: u32,\n    active_job: Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'pub fn extract_audio_to_mp3(\n    input: &Path,\n    output: &Path,\n    bitrate: u32,\n    active_job: Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n) -> Result<(), String> {\n    let total_duration = get_video_duration(input);'
    )
    content = content.replace(
        'wait_for_ffmpeg(child, output, &active_job)',
        'wait_for_ffmpeg(child, output, &active_job, app, total_duration)'
    )
    
    # 2. convert_video
    content = content.replace(
        'pub fn convert_video(\n    input: &Path,\n    output: &Path,\n    crf: u8,\n    use_h265: bool,\n    active_job: Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'pub fn convert_video(\n    input: &Path,\n    output: &Path,\n    crf: u8,\n    use_h265: bool,\n    active_job: Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n) -> Result<(), String> {\n    let total_duration = get_video_duration(input);'
    )

    # 3. optimize_mp4
    content = content.replace(
        'pub fn optimize_mp4(\n    input: &Path,\n    output: &Path,\n    crf: u8,\n    use_h265: bool,\n    active_job: Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'pub fn optimize_mp4(\n    input: &Path,\n    output: &Path,\n    crf: u8,\n    use_h265: bool,\n    active_job: Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n) -> Result<(), String> {\n    let total_duration = get_video_duration(input);'
    )

    # 4. convert_gif_to_mp4
    content = content.replace(
        'pub fn convert_gif_to_mp4(\n    input: &Path,\n    output: &Path,\n    active_job: Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'pub fn convert_gif_to_mp4(\n    input: &Path,\n    output: &Path,\n    active_job: Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n) -> Result<(), String> {\n    let total_duration = get_video_duration(input);'
    )

    # 5. convert_wav_to_mp3
    content = content.replace(
        'pub fn convert_wav_to_mp3(\n    input: &Path,\n    output: &Path,\n    bitrate: u32,\n    active_job: Arc<crate::ActiveJob>,\n) -> Result<(), String> {',
        'pub fn convert_wav_to_mp3(\n    input: &Path,\n    output: &Path,\n    bitrate: u32,\n    active_job: Arc<crate::ActiveJob>,\n    app: &tauri::AppHandle,\n) -> Result<(), String> {\n    let total_duration = get_video_duration(input);'
    )

    with open('src-tauri/src/optimizer/video.rs', 'w', encoding='utf-8') as f:
        f.write(content)

if __name__ == '__main__':
    main()
