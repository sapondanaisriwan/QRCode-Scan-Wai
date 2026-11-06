//! Memory-only capture. No IPC, decoder, filesystem access or continuous capture.
pub mod workflow;
use image::RgbaImage;
use serde::Serialize;

#[cfg(target_os = "windows")]
mod native;
#[cfg(target_os = "windows")]
pub use native::{capture_primary, desktop_metrics, wait_for_repaint};

pub const MAX_CAPTURE_PIXELS: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayMetrics {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale: f32,
    pub primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureError {
    CaptureFailed,
    Busy,
    WindowFailed,
    UnsupportedPlatform,
    CaptureDenied,
    DisplayChanged,
    DisplayUnavailable,
    ImageTooLarge,
}

pub struct CapturedFrame {
    pub display: DisplayMetrics,
    pub topology: Vec<DisplayMetrics>,
    pub pixels: RgbaImage,
}

pub fn validate_display(display: &DisplayMetrics) -> Result<(), CaptureError> {
    if display.width == 0
        || display.height == 0
        || !display.scale.is_finite()
        || display.scale <= 0.0
        || !matches!(display.rotation, 0.0 | 90.0 | 180.0 | 270.0)
    {
        return Err(CaptureError::DisplayUnavailable);
    }
    if u64::from(display.width) * u64::from(display.height) > MAX_CAPTURE_PIXELS {
        return Err(CaptureError::ImageTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_before_native_allocation() {
        let mut display = DisplayMetrics {
            id: 1,
            x: -1920,
            y: -1080,
            width: 1920,
            height: 1080,
            rotation: 0.0,
            scale: 1.5,
            primary: true,
        };
        assert_eq!(validate_display(&display), Ok(()));
        display.scale = f32::NAN;
        assert_eq!(
            validate_display(&display),
            Err(CaptureError::DisplayUnavailable)
        );
        display.scale = 1.0;
        display.width = u32::MAX;
        assert_eq!(validate_display(&display), Err(CaptureError::ImageTooLarge));
        display.width = 0;
        assert_eq!(
            validate_display(&display),
            Err(CaptureError::DisplayUnavailable)
        );
    }
}
