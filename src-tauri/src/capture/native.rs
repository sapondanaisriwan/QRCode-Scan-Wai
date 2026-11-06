use super::{validate_display, CaptureError, CapturedFrame, DisplayMetrics};
use windows::Win32::{
    Graphics::Dwm::DwmFlush,
    UI::HiDpi::{
        SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    },
};
use xcap::{Monitor, XCapError};

// This guard never crosses threads. Restore the caller's DPI context on every exit.
struct DpiGuard(DPI_AWARENESS_CONTEXT);
impl DpiGuard {
    fn enter() -> Result<Self, CaptureError> {
        // SAFETY: Windows-defined constant; the API only changes this calling thread.
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.0.is_null() {
            return Err(CaptureError::CaptureFailed);
        }
        Ok(Self(previous))
    }
}
impl Drop for DpiGuard {
    fn drop(&mut self) {
        // SAFETY: context was returned by Windows on this same thread.
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}
fn capture_error(error: XCapError) -> CaptureError {
    if let XCapError::WindowsCoreError(error) = error {
        if error.code().0 == 0x80070005_u32 as i32 {
            return CaptureError::CaptureDenied;
        }
    }
    CaptureError::CaptureFailed
}
fn metrics(monitor: &Monitor) -> Result<DisplayMetrics, CaptureError> {
    let display = DisplayMetrics {
        id: monitor.id().map_err(capture_error)?,
        x: monitor.x().map_err(capture_error)?,
        y: monitor.y().map_err(capture_error)?,
        width: monitor.width().map_err(capture_error)?,
        height: monitor.height().map_err(capture_error)?,
        rotation: monitor.rotation().map_err(capture_error)?,
        scale: monitor.scale_factor().map_err(capture_error)?,
        primary: monitor.is_primary().map_err(capture_error)?,
    };
    validate_display(&display)?;
    Ok(display)
}
/// Metadata only: never captures another frame.
pub fn desktop_metrics() -> Result<Vec<DisplayMetrics>, CaptureError> {
    let _dpi = DpiGuard::enter()?;
    let mut displays = Monitor::all()
        .map_err(capture_error)?
        .iter()
        .map(metrics)
        .collect::<Result<Vec<_>, _>>()?;
    displays.sort_by_key(|display| display.id);
    Ok(displays)
}
/// Call after hiding ScreenQR surfaces, from a blocking worker.
pub fn wait_for_repaint() -> Result<(), CaptureError> {
    // SAFETY: no pointers or owned resources; waits for the compositor's present.
    unsafe { DwmFlush() }.map_err(|_| CaptureError::CaptureFailed)?;
    std::thread::sleep(std::time::Duration::from_millis(120));
    Ok(())
}
/// Capture exactly one primary-display frame. No image or payload is logged/saved.
pub fn capture_primary() -> Result<CapturedFrame, CaptureError> {
    let _dpi = DpiGuard::enter()?;
    let topology = desktop_metrics()?;
    let display = topology
        .iter()
        .find(|display| display.primary)
        .cloned()
        .ok_or(CaptureError::DisplayUnavailable)?;
    let monitor = Monitor::all()
        .map_err(capture_error)?
        .into_iter()
        .find(|monitor| monitor.id().ok() == Some(display.id))
        .ok_or(CaptureError::DisplayChanged)?;
    let pixels = monitor.capture_image().map_err(capture_error)?;
    if pixels.dimensions() != (display.width, display.height) || desktop_metrics()? != topology {
        return Err(CaptureError::DisplayChanged);
    }
    Ok(CapturedFrame {
        display,
        topology,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_errors_do_not_expose_details_or_mislabel_protection() {
        let denied =
            windows::core::Error::from_hresult(windows::core::HRESULT(0x80070005_u32 as i32));
        assert_eq!(
            capture_error(XCapError::WindowsCoreError(denied)),
            CaptureError::CaptureDenied
        );
        assert_eq!(
            capture_error(XCapError::new("private native detail")),
            CaptureError::CaptureFailed
        );
    }
}
