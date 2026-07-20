import re

def main():
    with open('src-tauri/src/job.rs', 'r', encoding='utf-8') as f:
        content = f.read()

    # Replace fs::copy for the main out_file_path fallback
    content = content.replace('fs::copy(&in_file_path, &out_file_path)', 'atomic_copy(&in_file_path, &out_file_path)')
    
    # Also replace in copy_fallback_helper
    content = content.replace('fs::copy(in_file_path, &fallback_target)', 'atomic_copy(in_file_path, &fallback_target)')

    # Add atomic_copy helper function
    if 'fn atomic_copy' not in content:
        content += """\nfn atomic_copy(src: &Path, dst: &Path) -> std::io::Result<u64> {
    let tmp = dst.with_extension("copy.fileforge.tmp");
    let size = fs::copy(src, &tmp)?;
    let _ = fs::rename(&tmp, dst);
    Ok(size)
}\n"""

    def repl_mp3(m):
        txt = m.group(0)
        txt = txt.replace('let mp3_out = out_file_path.with_extension("mp3");',
            'let mp3_out = out_file_path.with_extension("mp3");\n                    let mp3_tmp = out_file_path.with_extension("mp3.fileforge.tmp");')
        txt = txt.replace('&mp3_out,', '&mp3_tmp,')
        txt = txt.replace('Ok(_) => {', 'Ok(_) => {\n                            let _ = fs::rename(&mp3_tmp, &mp3_out);')
        txt = txt.replace('Err(ref e) if e == "Скасовано" => return,', 'Err(ref e) if e == "Скасовано" => {\n                            let _ = fs::remove_file(&mp3_tmp);\n                            return;\n                        }')
        txt = txt.replace('Err(ref e) if e == "Скасовано" => {', 'Err(ref e) if e == "Скасовано" => {\n                            let _ = fs::remove_file(&mp3_tmp);')
        txt = txt.replace('Err(_) => {', 'Err(_) => {\n                            let _ = fs::remove_file(&mp3_tmp);')
        return txt

    def repl_mp4(m):
        txt = m.group(0)
        txt = txt.replace('let mp4_out = out_file_path.with_extension("mp4");',
            'let mp4_out = out_file_path.with_extension("mp4");\n                    let mp4_tmp = out_file_path.with_extension("mp4.fileforge.tmp");')
        txt = txt.replace('&mp4_out,', '&mp4_tmp,')
        txt = txt.replace('Ok(_) => {', 'Ok(_) => {\n                            let _ = fs::rename(&mp4_tmp, &mp4_out);')
        txt = txt.replace('Err(ref e) if e == "Скасовано" => {', 'Err(ref e) if e == "Скасовано" => {\n                            let _ = fs::remove_file(&mp4_tmp);')
        txt = txt.replace('Err(_) => {', 'Err(_) => {\n                            let _ = fs::remove_file(&mp4_tmp);')
        return txt

    content = re.sub(r'if options\.extract_audio \{.*?let _ = app\.emit\("job-progress".*?;\n\s+return;\n\s+\}', repl_mp3, content, flags=re.DOTALL)
    content = re.sub(r'if options\.convert_video \{.*?let _ = app\.emit\("job-progress".*?;\n\s+return;\n\s+\}', repl_mp4, content, flags=re.DOTALL)
    content = re.sub(r'if options\.convert_gif_to_mp4 \{.*?let _ = app\.emit\("job-progress".*?;\n\s+return;\n\s+\}', repl_mp4, content, flags=re.DOTALL)
    content = re.sub(r'if options\.convert_wav_to_mp3 \{.*?let _ = app\.emit\("job-progress".*?;\n\s+return;\n\s+\}', repl_mp3, content, flags=re.DOTALL)

    with open('src-tauri/src/job.rs', 'w', encoding='utf-8') as f:
        f.write(content)

if __name__ == '__main__':
    main()
