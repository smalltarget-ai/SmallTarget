use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, load_from_memory, DynamicImage, GenericImageView, ImageReader};
use std::io::Cursor;

/// open image from path
pub fn image_from_path(path: &str) -> Result<DynamicImage> {
    let img = ImageReader::open(&path)?.decode()?;
    Ok(img)
}

fn convert_to_jpeg_in_memory(dynamic_img: DynamicImage) -> image::ImageResult<DynamicImage> {
    let rgb_img = dynamic_img.to_rgb8();

    // 創建內存緩衝區
    let mut buffer = Cursor::new(Vec::new());

    // 編碼為JPEG
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 80);
    encoder.encode_image(&rgb_img)?;
    
    // 從JPEG字節流重新解碼為DynamicImage
    let jpeg_data = buffer.into_inner();
    let image = load_from_memory(&jpeg_data)?;
    Ok(image)
}

/// resize image to match max_pixels
pub fn image_resize(image: DynamicImage, max_pixels: u32) -> Result<DynamicImage> {
    let (width, height) = image.dimensions();
    let current_pixels = width * height;
    let (mut resized_width, mut resized_height) = (width, height);
    if current_pixels > max_pixels {
        // calculate resize factor
        let resize_factor = (max_pixels as f64 / current_pixels as f64).sqrt();
        resized_width = (width as f64 * resize_factor).floor() as u32;
        resized_height = (height as f64 * resize_factor).floor() as u32;
    }
    // resize image
    let resized_image = image.resize(resized_width, resized_height, FilterType::CatmullRom);
    //convert to jpeg and return DynamicImage
    let image = convert_to_jpeg_in_memory(resized_image)?;
    Ok(image)
}

// convert image to base64
pub fn image_to_base64(image: DynamicImage) -> Result<String> {
    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;
    let image_base64 = format!("data:image/png;base64,{}", STANDARD.encode(bytes));
    Ok(image_base64)
}
