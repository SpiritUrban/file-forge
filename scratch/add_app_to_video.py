import re

def main():
    with open('src-tauri/src/job.rs', 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 1. extract_audio_to_mp3
    content = content.replace('options.mp3_bitrate,\n                        active_job.clone(),',
                              'options.mp3_bitrate,\n                        active_job.clone(),\n                        &app,')
    content = content.replace('options.mp3_bitrate,\n                            active_job.clone(),',
                              'options.mp3_bitrate,\n                            active_job.clone(),\n                            &app,')

    # 2. convert_video
    content = content.replace('options.use_h265,\n                        active_job.clone(),',
                              'options.use_h265,\n                        active_job.clone(),\n                        &app,')
    
    # 3. optimize_mp4
    content = content.replace('options.use_h265,\n                            active_job.clone(),',
                              'options.use_h265,\n                            active_job.clone(),\n                            &app,')
    
    # 4. convert_gif_to_mp4
    content = content.replace('&mp4_tmp,\n                        active_job.clone(),',
                              '&mp4_tmp,\n                        active_job.clone(),\n                        &app,')
    
    # 5. convert_wav_to_mp3
    content = content.replace('options.mp3_bitrate,\n                        active_job.clone(),',
                              'options.mp3_bitrate,\n                        active_job.clone(),\n                        &app,')

    with open('src-tauri/src/job.rs', 'w', encoding='utf-8') as f:
        f.write(content)

if __name__ == '__main__':
    main()
