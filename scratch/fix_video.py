import re

def main():
    # Fix video.rs wait_for_ffmpeg calls
    with open('src-tauri/src/optimizer/video.rs', 'r', encoding='utf-8') as f:
        content = f.read()
    
    content = content.replace('wait_for_ffmpeg(child, output_path, &active_job)', 
                              'wait_for_ffmpeg(child, output_path, &active_job, app, total_duration)')
    content = content.replace('wait_for_ffmpeg(child, output, &active_job)', 
                              'wait_for_ffmpeg(child, output, &active_job, app, total_duration)')
    
    with open('src-tauri/src/optimizer/video.rs', 'w', encoding='utf-8') as f:
        f.write(content)

    # Fix job.rs copy_fallback_helper calls which incorrectly got &app
    with open('src-tauri/src/job.rs', 'r', encoding='utf-8') as f:
        job_content = f.read()

    job_content = job_content.replace('active_job.clone(), &app);', 'active_job.clone());')
    job_content = job_content.replace('active_job.clone(), &app\n                            );', 'active_job.clone()\n                            );')
    job_content = job_content.replace('active_job.clone(),\n                            &app,', 'active_job.clone(),')
    job_content = job_content.replace('active_job.clone(),\n                                &app,', 'active_job.clone(),')
    job_content = job_content.replace('active_job.clone(),\n                                    &app,', 'active_job.clone(),')
    job_content = job_content.replace('active_job.clone(),\n                        &app,', 'active_job.clone(),')
    job_content = job_content.replace('active_job.clone(),\n                    &app,', 'active_job.clone(),')
    
    # Wait, in the errors:
    # 533 - active_job.clone(),
    # 534 - &app,
    # Let's restore the video function calls to explicitly add &app ONLY to the right ones.
    
    # Let's just fix the job.rs by undoing the regex and doing it correctly.
    # Actually, I'll just check out job.rs from git again and re-apply cleanly.
    pass

if __name__ == '__main__':
    main()
