use image::codecs::webp::WebPEncoder;
use std::fs::File;
use std::path::Path;

pub fn optimize_webp(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let img = image::open(input_path).map_err(|e| e.to_string())?;
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let encoder = WebPEncoder::new_lossless(file);
    encoder
        .encode(
            img.as_bytes(),
            img.width(),
            img.height(),
            img.color().into(),
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
