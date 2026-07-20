import re

def main():
    # 1. Fix video.rs
    with open('src-tauri/src/optimizer/video.rs', 'r', encoding='utf-8') as f:
        v_content = f.read()

    # Add app parameter to all public functions returning Result<(), String>
    # convert_video
    v_content = re.sub(
        r'(pub fn convert_video\([\s\S]*?active_job: Arc<crate::ActiveJob>,\n)',
        r'\1    app: &tauri::AppHandle,\n',
        v_content
    )
    # optimize_mp4
    v_content = re.sub(
        r'(pub fn optimize_mp4\([\s\S]*?active_job: Arc<crate::ActiveJob>,\n)',
        r'\1    app: &tauri::AppHandle,\n',
        v_content
    )
    # convert_wav_to_mp3
    v_content = re.sub(
        r'(pub fn convert_wav_to_mp3\([\s\S]*?active_job: Arc<crate::ActiveJob>,\n)',
        r'\1    app: &tauri::AppHandle,\n',
        v_content
    )
    # extract_audio_to_mp3
    v_content = re.sub(
        r'(pub fn extract_audio_to_mp3\([\s\S]*?active_job: Arc<crate::ActiveJob>,\n)',
        r'\1    app: &tauri::AppHandle,\n',
        v_content
    )
    # convert_gif_to_mp4
    v_content = re.sub(
        r'(pub fn convert_gif_to_mp4\([\s\S]*?active_job: Arc<crate::ActiveJob>,\n)',
        r'\1    app: &tauri::AppHandle,\n',
        v_content
    )

    # Insert `let total_duration = get_video_duration(input_path);` right after `) -> Result<(), String> {`
    # for each of these functions.
    def inject_total_dur(m):
        return m.group(1) + '    let total_duration = get_video_duration(input_path);\n'
    v_content = re.sub(r'(\) -> Result<\(\), String> \{\n)', inject_total_dur, v_content)

    with open('src-tauri/src/optimizer/video.rs', 'w', encoding='utf-8') as f:
        f.write(v_content)
        
    # 2. Fix job.rs
    with open('src-tauri/src/job.rs', 'r', encoding='utf-8') as f:
        j_content = f.read()

    # In job.rs we have errors because of EXTRA arguments.
    # We replaced `active_job.clone(),` with `active_job.clone(), &app,`. But some were duplicated.
    # Let's clean up any `&app,` in job.rs and then re-add it carefully.
    j_content = re.sub(r'&app,\s*', '', j_content)
    
    # Now add it back exactly where needed
    j_content = re.sub(r'(crate::optimizer::video::extract_audio_to_mp3\([\s\S]*?active_job\.clone\(\)),', 
                       r'\1, &app,', j_content)
    j_content = re.sub(r'(crate::optimizer::video::convert_video\([\s\S]*?active_job\.clone\(\)),', 
                       r'\1, &app,', j_content)
    j_content = re.sub(r'(crate::optimizer::video::optimize_mp4\([\s\S]*?active_job\.clone\(\)),', 
                       r'\1, &app,', j_content)
    j_content = re.sub(r'(crate::optimizer::video::convert_wav_to_mp3\([\s\S]*?active_job\.clone\(\)),', 
                       r'\1, &app,', j_content)
    j_content = re.sub(r'(crate::optimizer::video::convert_gif_to_mp4\([\s\S]*?active_job\.clone\(\)),', 
                       r'\1, &app,', j_content)
                       
    with open('src-tauri/src/job.rs', 'w', encoding='utf-8') as f:
        f.write(j_content)

if __name__ == '__main__':
    main()
