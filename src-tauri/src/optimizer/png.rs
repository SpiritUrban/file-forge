use oxipng::{optimize, InFile, Options, OutFile};
use std::path::Path;

pub fn optimize_png(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let options = Options::from_preset(2);
    optimize(
        &InFile::Path(input_path.to_path_buf()),
        &OutFile::Path {
            path: Some(output_path.to_path_buf()),
            preserve_attrs: false,
        },
        &options,
    )
    .map_err(|e| e.to_string())
}
