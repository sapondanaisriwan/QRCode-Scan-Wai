use crate::capture::{
    workflow::{with_hidden_surface, CaptureSurface},
    CaptureError,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

static CAPTURING: AtomicBool = AtomicBool::new(false);
pub(super) struct Permit;
impl Permit {
    pub(super) fn acquire() -> Result<Self, CaptureError> {
        CAPTURING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| CaptureError::Busy)
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        CAPTURING.store(false, Ordering::Release);
    }
}
#[derive(Debug, Serialize)]
pub struct CaptureFailure {
    code: CaptureError,
    message: &'static str,
}
impl From<CaptureError> for CaptureFailure {
    fn from(code: CaptureError) -> Self {
        let message = match code {
            CaptureError::Busy => "A capture is already running. Please wait.",
            CaptureError::WindowFailed => "Cannot hide or restore ScreenQR. Show it from the taskbar and retry.",
            CaptureError::CaptureDenied => "Windows denied screen capture. Return to an unlocked desktop and retry.",
            CaptureError::DisplayChanged => "Display settings changed. Retry capture.",
            CaptureError::DisplayUnavailable => "The primary display is unavailable. Reconnect it and retry.",
            CaptureError::ImageTooLarge => "This display exceeds the capture size limit.",
            CaptureError::UnsupportedPlatform => "Screen capture requires Windows.",
            CaptureError::CaptureFailed => "Cannot capture this desktop. Return to an unlocked desktop and retry. Protected content may be blank.",
        };
        Self { code, message }
    }
}
pub(super) struct MainSurface(pub(super) tauri::WebviewWindow);
impl CaptureSurface for MainSurface {
    fn hide(&self) -> Result<(), CaptureError> {
        self.0.hide().map_err(|_| CaptureError::WindowFailed)?;
        // Query is dispatched after hide, so capture cannot race a queued window operation.
        if self
            .0
            .is_visible()
            .map_err(|_| CaptureError::WindowFailed)?
        {
            return Err(CaptureError::WindowFailed);
        }
        Ok(())
    }
    fn restore(&self) -> Result<(), CaptureError> {
        self.0.show().map_err(|_| CaptureError::WindowFailed)?;
        self.0.set_focus().map_err(|_| CaptureError::WindowFailed)
    }
}
fn preview() -> Result<Vec<u8>, CaptureError> {
    #[cfg(target_os = "windows")]
    {
        use std::io::Cursor;
        crate::capture::wait_for_repaint()?;
        let frame = crate::capture::capture_primary()?;
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(frame.pixels)
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|_| CaptureError::CaptureFailed)?;
        Ok(png.into_inner())
    }
    #[cfg(not(target_os = "windows"))]
    Err(CaptureError::UnsupportedPlatform)
}
/// Temporary phase-3.1 diagnostic. No paths, URLs, image inputs or persistent session.
#[tauri::command]
pub async fn capture_preview(
    window: tauri::WebviewWindow,
) -> Result<tauri::ipc::Response, CaptureFailure> {
    if window.label() != "main" {
        return Err(CaptureError::WindowFailed.into());
    }
    let permit = Permit::acquire()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        with_hidden_surface(&MainSurface(window), preview)
    })
    .await
    .map_err(|_| CaptureFailure::from(CaptureError::CaptureFailed))?
    .map(tauri::ipc::Response::new)
    .map_err(Into::into)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_capture_is_rejected_and_permit_released() {
        let permit = Permit::acquire().unwrap();
        assert!(matches!(Permit::acquire(), Err(CaptureError::Busy)));
        drop(permit);
        assert!(Permit::acquire().is_ok());
    }
    #[test]
    fn error_contract_contains_code_and_safe_message() {
        let value =
            serde_json::to_value(CaptureFailure::from(CaptureError::CaptureDenied)).unwrap();
        assert_eq!(value["code"], "capture_denied");
        assert!(value["message"]
            .as_str()
            .unwrap()
            .contains("Windows denied"));
    }
}
