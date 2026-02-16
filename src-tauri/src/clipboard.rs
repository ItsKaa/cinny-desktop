use arboard::{Clipboard};
use base64::{engine::general_purpose, Engine as _};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::io::Cursor;

#[tauri::command]
pub fn clipboard_read_image() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let image_data = clipboard.get_image().map_err(|e| e.to_string())?;

    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
        image_data.width.try_into().map_err(|_| "Invalid width".to_string())?,
        image_data.height.try_into().map_err(|_| "Invalid height".to_string())?,
        image_data.bytes.into_owned(),
    ).ok_or("Failed to create image buffer from clipboard data")?;

    // Handle fully transparent images for Windows (Windows uses DIB)
    // and force to opaque when it's fully transparent.
    let mut all_alpha_zero = true;
    for pixel in img.pixels() {
        if pixel[3] != 0 {
            all_alpha_zero = false;
            break;
        }
    }

    if all_alpha_zero {
        for pixel in img.pixels_mut() {
            pixel[3] = 255;
        }
    }

    // Convert to WEBP with fixed quality percentage.
    let dyn_img = DynamicImage::ImageRgba8(img);
    let encoder = webp::Encoder::from_image(&dyn_img)
        .map_err(|e| format!("Failed to create a webp encoder: {}", e))?;
    let mut webp_data = encoder.encode(95.0);

    let base64_str = general_purpose::STANDARD_NO_PAD.encode(&webp_data[..]);

    Ok(base64_str)
}