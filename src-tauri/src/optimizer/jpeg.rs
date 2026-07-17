use exif::{In, Reader, Tag};
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn get_exif_orientation(path: &Path) -> Option<u32> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let exif_reader = Reader::new();
    let exif = exif_reader.read_from_container(&mut reader).ok()?;
    let orientation = exif.get_field(Tag::Orientation, In::PRIMARY)?;
    orientation.value.get_uint(0)
}

pub fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        1 => img,
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.fliph().rotate270(),
        6 => img.rotate90(),
        7 => img.fliph().rotate90(),
        8 => img.rotate270(),
        _ => img,
    }
}

pub fn optimize_jpeg(input_path: &Path, output_path: &Path, quality: u8) -> Result<(), String> {
    // Determine orientation (fail-safe to 1/normal if not present)
    let orientation = get_exif_orientation(input_path).unwrap_or(1);

    // Open the image
    let img = image::open(input_path).map_err(|e| e.to_string())?;

    // Correct the orientation if necessary
    let oriented_img = apply_orientation(img, orientation);

    // Write back optimized JPEG at custom quality
    let file = File::create(output_path).map_err(|e| e.to_string())?;
    let mut encoder = JpegEncoder::new_with_quality(file, quality);
    encoder
        .encode_image(&oriented_img)
        .map_err(|e| e.to_string())?;

    Ok(())
}
