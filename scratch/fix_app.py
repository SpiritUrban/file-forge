import re

def main():
    with open('src-tauri/src/optimizer/video.rs', 'r', encoding='utf-8') as f:
        v_content = f.read()

    v_content = v_content.replace('wait_for_ffmpeg(child, output_path, &active_job)', 
                                  'wait_for_ffmpeg(child, output_path, &active_job, app, total_duration)')
    v_content = v_content.replace('wait_for_ffmpeg(child, output, &active_job)', 
                                  'wait_for_ffmpeg(child, output, &active_job, app, total_duration)')
    with open('src-tauri/src/optimizer/video.rs', 'w', encoding='utf-8') as f:
        f.write(v_content)

    with open('src-tauri/src/job.rs', 'r', encoding='utf-8') as f:
        j_content = f.read()

    j_content = j_content.replace('&app,\n                            &app,', '&app,')
    j_content = j_content.replace('&app,\n                        &app,', '&app,')
    
    with open('src-tauri/src/job.rs', 'w', encoding='utf-8') as f:
        f.write(j_content)

if __name__ == '__main__':
    main()
