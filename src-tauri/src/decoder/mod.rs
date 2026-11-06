//! Standalone image-to-result service. No Tauri, filesystem, or network access.
pub mod content;
pub mod types;

use image::{ImageFormat, ImageReader, Limits};
use std::io::Cursor;
use types::{DecodeBatch, DecodeError, ErrorCode};

pub const MAX_ENCODED_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_DIMENSION: u32 = 4096;
pub const MAX_PIXELS: u64 = 4 * 1024 * 1024;

fn image_error(error: image::ImageError) -> DecodeError {
    DecodeError::new(if matches!(error, image::ImageError::Limits(_)) {
        ErrorCode::ImageTooLarge
    } else {
        ErrorCode::InvalidImage
    })
}

pub fn decode_image(bytes: &[u8]) -> Result<DecodeBatch, DecodeError> {
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(DecodeError::new(ErrorCode::ImageTooLarge));
    }
    if image::guess_format(bytes).ok() != Some(ImageFormat::Png) {
        return Err(DecodeError::new(ErrorCode::InvalidImage));
    }
    let reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let (width, height) = reader.into_dimensions().map_err(image_error)?;
    if width == 0 || height == 0 {
        return Err(DecodeError::new(ErrorCode::InvalidImage));
    }
    if width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(DecodeError::new(ErrorCode::ImageTooLarge));
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let gray = reader.decode().map_err(image_error)?.into_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(gray);
    let mut grids = prepared.detect_grids();
    if grids.is_empty() {
        return Err(DecodeError::new(ErrorCode::NoQrDetected));
    }
    // Stable spatial order independent of detector enumeration; duplicates remain distinct.
    grids.sort_by_key(|grid| {
        let top = grid.bounds.iter().map(|p| p.y).min().unwrap_or(0);
        let left = grid.bounds.iter().map(|p| p.x).min().unwrap_or(0);
        (top, left)
    });
    let mut batch = DecodeBatch {
        results: Vec::new(),
        unreadable_count: 0,
        unsupported_count: 0,
    };
    for grid in grids {
        let mut payload = Vec::new();
        if grid.decode_to(&mut payload).is_err() {
            batch.unreadable_count += 1;
            continue;
        }
        match String::from_utf8(payload) {
            Ok(content) => batch.results.push(content::classify(content)),
            Err(_) => batch.unsupported_count += 1,
        }
    }
    if batch.results.is_empty() {
        return Err(DecodeError::new(if batch.unreadable_count > 0 {
            ErrorCode::MalformedQr
        } else {
            ErrorCode::UnsupportedContent
        }));
    }
    Ok(batch)
}
