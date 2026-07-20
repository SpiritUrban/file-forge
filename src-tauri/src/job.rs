use crate::models::{JobProgress, JobStatus};
use crate::optimizer::jpeg::optimize_jpeg;
use crate::optimizer::png::optimize_png;
use crate::scanner::scan_directory;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct ActiveJob {
    pub progress: Mutex<JobProgress>,
    pub running: Mutex<bool>,
    pub cancelled: AtomicBool,
}

impl Default for ActiveJob {
    fn default() -> Self {
        Self::new()
    }
}

impl ActiveJob {
    pub fn new() -> Self {
        Self {
            progress: Mutex::new(JobProgress::default()),
            running: Mutex::new(false),
            cancelled: AtomicBool::new(false),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    Jpeg,
    Png,
    Webp,
    Svg,
    VideoVob,
    VideoAvi,
    VideoMkv,
    VideoMov,
    VideoWmv,
    VideoFlv,
    VideoWebm,
    VideoMp4,
    AudioWav,
    AudioFlac,
    AudioOgg,
    AudioM4a,
    ImageBmp,
    ImageTiff,
    ImageGif,
    Other,
}

pub fn classify_file(path: &Path) -> FileType {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        if ext_lower == "jpg" || ext_lower == "jpeg" {
            FileType::Jpeg
        } else if ext_lower == "png" {
            FileType::Png
        } else if ext_lower == "webp" {
            FileType::Webp
        } else if ext_lower == "svg" {
            FileType::Svg
        } else if ext_lower == "vob" {
            FileType::VideoVob
        } else if ext_lower == "avi" {
            FileType::VideoAvi
        } else if ext_lower == "mkv" {
            FileType::VideoMkv
        } else if ext_lower == "mov" {
            FileType::VideoMov
        } else if ext_lower == "wmv" {
            FileType::VideoWmv
        } else if ext_lower == "flv" {
            FileType::VideoFlv
        } else if ext_lower == "webm" {
            FileType::VideoWebm
        } else if ext_lower == "mp4" {
            FileType::VideoMp4
        } else if ext_lower == "wav" {
            FileType::AudioWav
        } else if ext_lower == "flac" {
            FileType::AudioFlac
        } else if ext_lower == "ogg" {
            FileType::AudioOgg
        } else if ext_lower == "m4a" {
            FileType::AudioM4a
        } else if ext_lower == "bmp" {
            FileType::ImageBmp
        } else if ext_lower == "tiff" || ext_lower == "tif" {
            FileType::ImageTiff
        } else if ext_lower == "gif" {
            FileType::ImageGif
        } else {
            FileType::Other
        }
    } else {
        FileType::Other
    }
}

pub fn validate_paths(input: &Path, output: &Path) -> Result<(), String> {
    if !input.exists() {
        return Err("Вхідний шлях не існує.".to_string());
    }
    if !input.is_dir() {
        return Err("Вхідний шлях не є папкою.".to_string());
    }
    if fs::read_dir(input).is_err() {
        return Err("Не вдалося прочитати вхідну папку (відсутній доступ).".to_string());
    }
    if input == output {
        return Err("Вихідна папка не повинна збігатися з вхідною.".to_string());
    }
    if output.starts_with(input) {
        return Err("Вихідна папка не повинна знаходитися всередині вхідної.".to_string());
    }
    Ok(())
}

pub fn run_optimization_job(
    app: AppHandle,
    input_path: PathBuf,
    output_path: PathBuf,
    active_job: Arc<crate::ActiveJob>,
    options: crate::models::JobOptions,
) {
    // 1. Set status to scanning and reset cancelled flag
    {
        let mut progress = active_job.progress.lock().unwrap();
        *progress = JobProgress::default();
        progress.status = JobStatus::Scanning;
        active_job.cancelled.store(false, Ordering::Relaxed);
    }
    let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());

    // 2. Validate paths
    if let Err(err) = validate_paths(&input_path, &output_path) {
        {
            let mut progress = active_job.progress.lock().unwrap();
            progress.status = JobStatus::Failed;
        }
        let _ = app.emit("job-error", err);
        let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
        let mut running = active_job.running.lock().unwrap();
        *running = false;
        return;
    }

    // 3. Scan input folder
    let scanned = match scan_directory(&input_path) {
        Ok(data) => data,
        Err(err) => {
            {
                let mut progress = active_job.progress.lock().unwrap();
                progress.status = JobStatus::Failed;
            }
            let _ = app.emit("job-error", format!("Помилка сканування: {}", err));
            let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
            let mut running = active_job.running.lock().unwrap();
            *running = false;
            return;
        }
    };

    // 4. Update status to processing
    {
        let mut progress = active_job.progress.lock().unwrap();
        progress.status = JobStatus::Processing;
        progress.total_files = scanned.files.len();
        progress.original_bytes = scanned.total_bytes;
    }
    let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());

    // 5. Create output path and subfolders
    if let Err(e) = fs::create_dir_all(&output_path) {
        {
            let mut progress = active_job.progress.lock().unwrap();
            progress.status = JobStatus::Failed;
        }
        let _ = app.emit(
            "job-error",
            format!("Не вдалося створити вихідну папку: {}", e),
        );
        let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
        let mut running = active_job.running.lock().unwrap();
        *running = false;
        return;
    }

    for dir in &scanned.dirs {
        let target_dir = output_path.join(dir);
        if let Err(e) = fs::create_dir_all(&target_dir) {
            println!(
                "Warning: failed to create subfolder {:?}: {:?}",
                target_dir, e
            );
        }
    }

    // 6. Process files concurrently
    scanned.files.par_iter().for_each(|(rel_path, orig_size)| {
        if active_job.cancelled.load(Ordering::Relaxed) {
            return;
        }

        let in_file_path = input_path.join(rel_path);
        let file_type = classify_file(&in_file_path);

        let is_png_to_webp = file_type == FileType::Png && options.convert_png_to_webp;
        let out_rel_path = if is_png_to_webp {
            rel_path.with_extension("webp")
        } else {
            rel_path.clone()
        };
        let out_file_path = output_path.join(&out_rel_path);

        let mut final_out_file_path = out_file_path.clone();
        match file_type {
            FileType::VideoVob | FileType::VideoAvi | FileType::VideoMkv | FileType::VideoMov | FileType::VideoWmv | FileType::VideoFlv | FileType::VideoWebm => {
                if options.extract_audio {
                    final_out_file_path = final_out_file_path.with_extension("mp3");
                } else if options.convert_video {
                    final_out_file_path = final_out_file_path.with_extension("mp4");
                }
            }
            FileType::VideoMp4 => {
                if options.extract_audio {
                    final_out_file_path = final_out_file_path.with_extension("mp3");
                }
            }
            FileType::AudioWav | FileType::AudioFlac | FileType::AudioOgg | FileType::AudioM4a => {
                if options.convert_wav_to_mp3 {
                    final_out_file_path = final_out_file_path.with_extension("mp3");
                }
            }
            FileType::ImageGif => {
                if options.convert_gif_to_mp4 {
                    final_out_file_path = final_out_file_path.with_extension("mp4");
                }
            }
            _ => {}
        }

        if final_out_file_path.exists() {
            let out_size = fs::metadata(&final_out_file_path).map(|m| m.len()).unwrap_or(0);
            {
                let mut progress = active_job.progress.lock().unwrap();
                progress.processed_files += 1;
                progress.skipped_files += 1;
                progress.output_bytes += out_size;
            }
            let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
            return;
        }

        // Safety fallback: ensure parent directory exists
        if let Some(parent) = out_file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Update current file in progress
        {
            let mut progress = active_job.progress.lock().unwrap();
            progress.current_file = Some(in_file_path.file_name().unwrap_or_default().to_string_lossy().to_string());
            progress.current_file_progress = None;
        }
        let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());

        let optimize_result = match file_type {
            FileType::Other => {
                if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                    let mut progress = active_job.progress.lock().unwrap();
                    progress.processed_files += 1;
                    progress.copied_files += 1;
                    progress.output_bytes += orig_size;
                } else {
                    let mut progress = active_job.progress.lock().unwrap();
                    progress.processed_files += 1;
                    progress.failed_files += 1;
                }
                return;
            }
            FileType::Jpeg => {
                let temp = out_file_path.with_extension("jpg.fileforge.tmp");
                if options.resize_images {
                    (
                        process_with_resize_and_optimize(
                            &in_file_path,
                            &temp,
                            FileType::Jpeg,
                            &options,
                        ),
                        temp,
                        false,
                    )
                } else {
                    (
                        optimize_jpeg(&in_file_path, &temp, options.jpeg_quality),
                        temp,
                        false,
                    )
                }
            }
            FileType::Png | FileType::ImageBmp | FileType::ImageTiff => {
                if options.convert_png_to_webp {
                    let temp = out_file_path.with_extension("webp.fileforge.tmp");
                    (
                        process_with_resize_and_optimize(
                            &in_file_path,
                            &temp,
                            FileType::Webp,
                            &options,
                        ),
                        temp,
                        true,
                    )
                } else if options.resize_images {
                    let temp = out_file_path.with_extension("png.fileforge.tmp");
                    (
                        process_with_resize_and_optimize(
                            &in_file_path,
                            &temp,
                            FileType::Png,
                            &options,
                        ),
                        temp,
                        false,
                    )
                } else {
                    let temp = out_file_path.with_extension("png.fileforge.tmp");
                    (optimize_png(&in_file_path, &temp), temp, false)
                }
            }
            FileType::Webp => {
                if options.optimize_webp {
                    let temp = out_file_path.with_extension("webp.fileforge.tmp");
                    if options.resize_images {
                        (
                            process_with_resize_and_optimize(
                                &in_file_path,
                                &temp,
                                FileType::Webp,
                                &options,
                            ),
                            temp,
                            false,
                        )
                    } else {
                        (
                            crate::optimizer::webp::optimize_webp(&in_file_path, &temp),
                            temp,
                            false,
                        )
                    }
                } else {
                    (
                        Err("webp optimization disabled".to_string()),
                        PathBuf::new(),
                        false,
                    )
                }
            }
            FileType::Svg => {
                if options.optimize_svg {
                    let temp = out_file_path.with_extension("svg.fileforge.tmp");
                    (
                        crate::optimizer::svg::optimize_svg(&in_file_path, &temp),
                        temp,
                        false,
                    )
                } else {
                    (
                        Err("svg optimization disabled".to_string()),
                        PathBuf::new(),
                        false,
                    )
                }
            }
            FileType::VideoVob | FileType::VideoAvi | FileType::VideoMkv | FileType::VideoMov | FileType::VideoWmv | FileType::VideoFlv | FileType::VideoWebm => {
                if options.extract_audio {
                    let mp3_out = out_file_path.with_extension("mp3");
                    let mp3_tmp = out_file_path.with_extension("mp3.fileforge.tmp");
                    if let Some(parent) = mp3_out.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let result = crate::optimizer::video::extract_audio_to_mp3(
                        &in_file_path,
                        &mp3_tmp,
                        options.mp3_bitrate,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let _ = fs::rename(&mp3_tmp, &mp3_out);
                            let out_size = fs::metadata(&mp3_out).map(|m| m.len()).unwrap_or(0);
                            let mut progress = active_job.progress.lock().unwrap();
                            progress.processed_files += 1;
                            progress.optimized_files += 1;
                            progress.output_bytes += out_size;
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&mp3_tmp);
                            let _ = fs::remove_file(&mp3_tmp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&mp3_tmp);
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            }
                        }
                    }
                    let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                    return;
                }
                if options.convert_video {
                    // Output is always .mp4 regardless of source extension
                    let mp4_out = out_file_path.with_extension("mp4");
                    let mp4_tmp = out_file_path.with_extension("mp4.fileforge.tmp");
                    if let Some(parent) = mp4_out.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let result = crate::optimizer::video::convert_video(
                        &in_file_path,
                        &mp4_tmp,
                        options.video_crf,
                        options.use_h265,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let _ = fs::rename(&mp4_tmp, &mp4_out);
                            let out_size = fs::metadata(&mp4_out).map(|m| m.len()).unwrap_or(0);
                            let mut progress = active_job.progress.lock().unwrap();
                            progress.processed_files += 1;
                            progress.optimized_files += 1;
                            progress.output_bytes += out_size;
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&mp4_tmp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&mp4_tmp);
                            // Conversion failed — copy original as fallback
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            } else {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.failed_files += 1;
                            }
                        }
                    }
                } else {
                    // Video conversion disabled — copy as-is
                    if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.copied_files += 1;
                        progress.output_bytes += orig_size;
                    } else {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.failed_files += 1;
                    }
                }
                let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                return;
            }
            FileType::VideoMp4 => {
                if options.extract_audio {
                    let mp3_out = out_file_path.with_extension("mp3");
                    let mp3_tmp = out_file_path.with_extension("mp3.fileforge.tmp");
                    if let Some(parent) = mp3_out.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let result = crate::optimizer::video::extract_audio_to_mp3(
                        &in_file_path,
                        &mp3_tmp,
                        options.mp3_bitrate,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let _ = fs::rename(&mp3_tmp, &mp3_out);
                            let out_size = fs::metadata(&mp3_out).map(|m| m.len()).unwrap_or(0);
                            let mut progress = active_job.progress.lock().unwrap();
                            progress.processed_files += 1;
                            progress.optimized_files += 1;
                            progress.output_bytes += out_size;
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&mp3_tmp);
                            let _ = fs::remove_file(&mp3_tmp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&mp3_tmp);
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            }
                        }
                    }
                    let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                    return;
                }

                if options.optimize_mp4 {
                    let temp = out_file_path.with_extension("mp4.fileforge.tmp");
                    let result = crate::optimizer::video::optimize_mp4(
                        &in_file_path,
                        &temp,
                        options.video_crf,
                        options.use_h265,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let out_size = fs::metadata(&temp).map(|m| m.len()).unwrap_or(0);
                            if out_size < *orig_size {
                                let _ = fs::rename(&temp, &out_file_path);
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.optimized_files += 1;
                                progress.output_bytes += out_size;
                            } else {
                                // Re-encoded file is larger — keep original
                                let _ = fs::remove_file(&temp);
                                if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                    let mut progress = active_job.progress.lock().unwrap();
                                    progress.processed_files += 1;
                                    progress.original_kept_files += 1;
                                    progress.output_bytes += orig_size;
                                }
                            }
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&temp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&temp);
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            }
                        }
                    }
                } else {
                    if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.copied_files += 1;
                        progress.output_bytes += orig_size;
                    } else {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.failed_files += 1;
                    }
                }
                let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                return;
            }
            FileType::AudioWav | FileType::AudioFlac | FileType::AudioOgg | FileType::AudioM4a => {
                if options.convert_wav_to_mp3 {
                    let mp3_out = out_file_path.with_extension("mp3");
                    let mp3_tmp = out_file_path.with_extension("mp3.fileforge.tmp");
                    if let Some(parent) = mp3_out.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let result = crate::optimizer::video::convert_wav_to_mp3(
                        &in_file_path,
                        &mp3_tmp,
                        options.mp3_bitrate,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let _ = fs::rename(&mp3_tmp, &mp3_out);
                            let out_size = fs::metadata(&mp3_out).map(|m| m.len()).unwrap_or(0);
                            let mut progress = active_job.progress.lock().unwrap();
                            progress.processed_files += 1;
                            progress.optimized_files += 1;
                            progress.output_bytes += out_size;
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&mp3_tmp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&mp3_tmp);
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            }
                        }
                    }
                } else {
                    if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.copied_files += 1;
                        progress.output_bytes += orig_size;
                    } else {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.failed_files += 1;
                    }
                }
                let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                return;
            }
            FileType::ImageGif => {
                if options.convert_gif_to_mp4 {
                    let mp4_out = out_file_path.with_extension("mp4");
                    let mp4_tmp = out_file_path.with_extension("mp4.fileforge.tmp");
                    if let Some(parent) = mp4_out.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let result = crate::optimizer::video::convert_gif_to_mp4(
                        &in_file_path,
                        &mp4_tmp,
                        active_job.clone(), &app,
                        );
                    match result {
                        Ok(_) => {
                            let _ = fs::rename(&mp4_tmp, &mp4_out);
                            let out_size = fs::metadata(&mp4_out).map(|m| m.len()).unwrap_or(0);
                            let mut progress = active_job.progress.lock().unwrap();
                            progress.processed_files += 1;
                            progress.optimized_files += 1;
                            progress.output_bytes += out_size;
                        }
                        Err(ref e) if e == "Скасовано" => {
                            let _ = fs::remove_file(&mp4_tmp);
                            return;
                        }
                        Err(_) => {
                            let _ = fs::remove_file(&mp4_tmp);
                            if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                                let mut progress = active_job.progress.lock().unwrap();
                                progress.processed_files += 1;
                                progress.copied_files += 1;
                                progress.output_bytes += orig_size;
                            }
                        }
                    }
                } else {
                    if atomic_copy(&in_file_path, &out_file_path).is_ok() {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.copied_files += 1;
                        progress.output_bytes += orig_size;
                    } else {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.failed_files += 1;
                    }
                }
                let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
                return;
            }
        };

        if active_job.cancelled.load(Ordering::Relaxed) {
            if optimize_result.1.exists() {
                let _ = fs::remove_file(&optimize_result.1);
            }
            return;
        }

        let (res, temp_file_path, always_keep) = optimize_result;

        match res {
            Ok(_) => {
                let temp_size = fs::metadata(&temp_file_path)
                    .map(|m| m.len())
                    .unwrap_or(u64::MAX);
                if always_keep || temp_size < *orig_size {
                    if let Err(e) = fs::rename(&temp_file_path, &out_file_path) {
                        println!("Failed to rename temp file: {:?}", e);
                        let _ = fs::remove_file(&temp_file_path);
                        copy_fallback_helper(
                            &in_file_path,
                            &out_file_path,
                            &output_path,
                            rel_path,
                            is_png_to_webp,
                            orig_size,
                            active_job.clone(),
                        );
                    } else {
                        let mut progress = active_job.progress.lock().unwrap();
                        progress.processed_files += 1;
                        progress.optimized_files += 1;
                        progress.output_bytes += temp_size;
                    }
                } else {
                    let _ = fs::remove_file(&temp_file_path);
                    copy_fallback_helper(
                        &in_file_path,
                        &out_file_path,
                        &output_path,
                        rel_path,
                        is_png_to_webp,
                        orig_size,
                        active_job.clone(),
                    );
                }
            }
            Err(_) => {
                if temp_file_path.exists() {
                    let _ = fs::remove_file(&temp_file_path);
                }
                copy_fallback_helper(
                    &in_file_path,
                    &out_file_path,
                    &output_path,
                    rel_path,
                    is_png_to_webp,
                    orig_size,
                    active_job.clone(),
                );
            }
        }
        let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());
    });

    // 7. Mark as completed or cancelled
    let was_cancelled = active_job.cancelled.load(Ordering::Relaxed);
    {
        let mut progress = active_job.progress.lock().unwrap();
        if was_cancelled {
            progress.status = JobStatus::Cancelled;
        } else {
            progress.status = JobStatus::Completed;
        }
        progress.current_file = None;
    }
    let _ = app.emit("job-progress", active_job.progress.lock().unwrap().clone());

    let mut running = active_job.running.lock().unwrap();
    *running = false;
}

fn process_with_resize_and_optimize(
    in_file_path: &Path,
    temp_file_path: &Path,
    file_type: FileType,
    options: &crate::models::JobOptions,
) -> Result<(), String> {
    // 1. Load image
    let img = image::open(in_file_path).map_err(|e| e.to_string())?;

    // 2. For JPEGs, apply EXIF orientation first so resizing happens on correct orientation
    let img = if file_type == FileType::Jpeg {
        let orientation = crate::optimizer::jpeg::get_exif_orientation(in_file_path).unwrap_or(1);
        crate::optimizer::jpeg::apply_orientation(img, orientation)
    } else {
        img
    };

    // 3. Resize if needed
    let img = if options.resize_images {
        if img.width() > options.max_width || img.height() > options.max_height {
            img.resize(
                options.max_width,
                options.max_height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            img
        }
    } else {
        img
    };

    // 4. Save to temp path depending on destination format
    match file_type {
        FileType::Jpeg => {
            let file = std::fs::File::create(temp_file_path).map_err(|e| e.to_string())?;
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(file, options.jpeg_quality);
            encoder.encode_image(&img).map_err(|e| e.to_string())?;
        }
        FileType::Png => {
            // Save as PNG
            img.save(temp_file_path).map_err(|e| e.to_string())?;
            // Optimize PNG with oxipng on the resized file
            let _ = crate::optimizer::png::optimize_png(temp_file_path, temp_file_path);
        }
        FileType::Webp => {
            let file = std::fs::File::create(temp_file_path).map_err(|e| e.to_string())?;
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(file);
            encoder
                .encode(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )
                .map_err(|e| e.to_string())?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn copy_fallback_helper(
    in_file_path: &Path,
    out_file_path: &Path,
    output_path: &Path,
    rel_path: &Path,
    is_png_to_webp: bool,
    orig_size: &u64,
    active_job: Arc<crate::ActiveJob>,
) {
    let fallback_target = if is_png_to_webp {
        output_path.join(rel_path)
    } else {
        out_file_path.to_path_buf()
    };
    if atomic_copy(in_file_path, &fallback_target).is_ok() {
        let mut progress = active_job.progress.lock().unwrap();
        progress.processed_files += 1;
        progress.original_kept_files += 1;
        progress.output_bytes += orig_size;
    } else {
        let mut progress = active_job.progress.lock().unwrap();
        progress.processed_files += 1;
        progress.failed_files += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_classify_file() {
        assert_eq!(classify_file(Path::new("photo.jpg")), FileType::Jpeg);
        assert_eq!(classify_file(Path::new("photo.JPEG")), FileType::Jpeg);
        assert_eq!(classify_file(Path::new("image.png")), FileType::Png);
        assert_eq!(classify_file(Path::new("image.webp")), FileType::Webp);
        assert_eq!(classify_file(Path::new("vector.svg")), FileType::Svg);
        assert_eq!(classify_file(Path::new("video.mp4")), FileType::VideoMp4);
        assert_eq!(classify_file(Path::new("audio.wav")), FileType::AudioWav);
        assert_eq!(classify_file(Path::new("audio.flac")), FileType::AudioFlac);
        assert_eq!(classify_file(Path::new("music.ogg")), FileType::AudioOgg);
        assert_eq!(classify_file(Path::new("podcast.m4a")), FileType::AudioM4a);
        assert_eq!(classify_file(Path::new("photo.bmp")), FileType::ImageBmp);
        assert_eq!(classify_file(Path::new("scan.tiff")), FileType::ImageTiff);
        assert_eq!(classify_file(Path::new("scan.tif")), FileType::ImageTiff);
        assert_eq!(classify_file(Path::new("anim.gif")), FileType::ImageGif);
        assert_eq!(classify_file(Path::new("no_extension")), FileType::Other);
    }

    #[test]
    fn test_classify_video() {
        assert_eq!(classify_file(Path::new("movie.vob")), FileType::VideoVob);
        assert_eq!(classify_file(Path::new("movie.VOB")), FileType::VideoVob);
        assert_eq!(classify_file(Path::new("clip.avi")), FileType::VideoAvi);
        assert_eq!(classify_file(Path::new("clip.AVI")), FileType::VideoAvi);
        assert_eq!(classify_file(Path::new("already.mp4")), FileType::VideoMp4);
        assert_eq!(classify_file(Path::new("film.mkv")), FileType::VideoMkv);
        assert_eq!(classify_file(Path::new("film.mov")), FileType::VideoMov);
        assert_eq!(classify_file(Path::new("film.wmv")), FileType::VideoWmv);
        assert_eq!(classify_file(Path::new("film.flv")), FileType::VideoFlv);
        assert_eq!(classify_file(Path::new("film.webm")), FileType::VideoWebm);
    }

    #[test]
    fn test_ffmpeg_check_does_not_panic() {
        // Should return a bool without panicking regardless of whether ffmpeg is installed
        let _ = crate::optimizer::video::ffmpeg_available();
    }

    #[test]
    fn test_svg_optimization() {
        let base_temp = std::env::temp_dir().join("file_forge_svg_test");
        let _ = fs::remove_dir_all(&base_temp);
        fs::create_dir_all(&base_temp).unwrap();

        let input_svg = base_temp.join("input.svg");
        let output_svg = base_temp.join("output.svg");

        let raw_svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Generated by SVG Editor -->
<svg width="100" height="100">
    <metadata>
        <editor:custom-tag>Some bloat metadata</editor:custom-tag>
    </metadata>
    <circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" />
</svg>"#;

        fs::write(&input_svg, raw_svg).unwrap();
        crate::optimizer::svg::optimize_svg(&input_svg, &output_svg).unwrap();

        let result_svg = fs::read_to_string(&output_svg).unwrap();
        assert!(!result_svg.contains("Generated by SVG Editor"));
        assert!(!result_svg.contains("<metadata>"));
        assert!(!result_svg.contains("Some bloat metadata"));
        assert!(result_svg.contains("<circle cx="));

        let _ = fs::remove_dir_all(&base_temp);
    }

    #[test]
    fn test_resize_bounds() {
        // Test that resize() proportionally scales down when either dimension exceeds limits.
        // We build a synthetic 800x600 image and verify boundaries are respected.
        let img = image::DynamicImage::new_rgb8(800, 600);

        let max_w = 400u32;
        let max_h = 400u32;

        let resized = if img.width() > max_w || img.height() > max_h {
            img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
        } else {
            img
        };

        assert!(
            resized.width() <= max_w,
            "Width exceeded max: {}",
            resized.width()
        );
        assert!(
            resized.height() <= max_h,
            "Height exceeded max: {}",
            resized.height()
        );

        // Check proportional: 800x600 bounded by 400x400 → should be 400x300
        assert_eq!(resized.width(), 400);
        assert_eq!(resized.height(), 300);
    }

    #[test]
    fn test_validate_paths() {
        let base_temp = std::env::temp_dir().join("file_forge_test_dir");
        let _ = fs::remove_dir_all(&base_temp);
        fs::create_dir_all(&base_temp).unwrap();

        let input = base_temp.join("input");
        let output = base_temp.join("output");

        fs::create_dir(&input).unwrap();

        // Success case
        assert!(validate_paths(&input, &output).is_ok());

        // Error: input does not exist
        let bad_input = base_temp.join("does_not_exist");
        assert_eq!(
            validate_paths(&bad_input, &output).unwrap_err(),
            "Вхідний шлях не існує."
        );

        // Error: input == output
        assert_eq!(
            validate_paths(&input, &input).unwrap_err(),
            "Вихідна папка не повинна збігатися з вхідною."
        );

        // Error: output inside input
        let nested_output = input.join("nested");
        assert_eq!(
            validate_paths(&input, &nested_output).unwrap_err(),
            "Вихідна папка не повинна знаходитися всередині вхідної."
        );

        // Error: output already exists
        fs::create_dir(&output).unwrap();
        assert!(validate_paths(&input, &output)
            .unwrap_err()
            .contains("Папка результату вже існує."));

        let _ = fs::remove_dir_all(&base_temp);
    }
}

fn atomic_copy(src: &Path, dst: &Path) -> std::io::Result<u64> {
    let tmp = dst.with_extension("copy.fileforge.tmp");
    let size = fs::copy(src, &tmp)?;
    let _ = fs::rename(&tmp, dst);
    Ok(size)
}
