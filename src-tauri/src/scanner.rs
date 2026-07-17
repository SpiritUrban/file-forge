use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ScannedData {
    pub files: Vec<(PathBuf, u64)>, // (relative_path, size_bytes)
    pub dirs: Vec<PathBuf>,         // relative_path
    pub total_bytes: u64,
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    (metadata.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub fn scan_directory(input_path: &Path) -> Result<ScannedData, String> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    let mut total_bytes = 0;

    let walker = WalkDir::new(input_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let path = entry.path();
            if path == input_path {
                return true;
            }
            match fs::symlink_metadata(path) {
                Ok(metadata) => {
                    if is_reparse_point(&metadata) {
                        println!("Skipping link/reparse point in scan: {:?}", path);
                        false
                    } else {
                        true
                    }
                }
                Err(e) => {
                    println!("Error reading metadata for {:?}: {:?}", path, e);
                    false
                }
            }
        });

    for entry_result in walker {
        let entry = entry_result.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path == input_path {
            continue;
        }

        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        let relative = path
            .strip_prefix(input_path)
            .map_err(|e| e.to_string())?
            .to_path_buf();

        if metadata.is_dir() {
            dirs.push(relative);
        } else if metadata.is_file() {
            let size = metadata.len();
            files.push((relative, size));
            total_bytes += size;
        }
    }

    // Sort to ensure deterministic order (important for progress reporting and testing)
    dirs.sort();
    files.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(ScannedData {
        files,
        dirs,
        total_bytes,
    })
}
