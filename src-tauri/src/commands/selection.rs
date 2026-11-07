//! Capture, select and decode native pixels; no screenshot persistence.
use super::capture::{CaptureFailure, MainSurface, Permit};
use crate::capture::crop::{self, PixelRect, Point};
use crate::capture::{self, workflow::with_hidden_surface, CaptureError, DisplayMetrics};
use crate::decoder::{
    self,
    types::{DecodeBatch, DecodeError, DecodeResponse, ErrorCode, QrBounds},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
#[derive(Default)]
pub struct SelectionState(Mutex<Option<Active>>);
struct Active {
    label: String,
    png: Vec<u8>,
    display: DisplayMetrics,
    sender: SyncSender<Action>,
    ready: bool,
    cancelled: Arc<AtomicBool>,
    scale: Option<f64>,
    review: Option<OverlayReview>,
}
enum Action {
    Cancel,
    Select(PixelRect),
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogicalRect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectionInput {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    viewport_width: f64,
    viewport_height: f64,
}
impl SelectionInput {
    fn normalize(&self, width: f64, height: f64) -> Result<Option<LogicalRect>, CaptureError> {
        if ![
            self.x1,
            self.y1,
            self.x2,
            self.y2,
            self.viewport_width,
            self.viewport_height,
            width,
            height,
        ]
        .iter()
        .all(|v| v.is_finite())
            || width <= 0.0
            || height <= 0.0
            || (self.viewport_width - width).abs() > 1.0
            || (self.viewport_height - height).abs() > 1.0
        {
            return Err(CaptureError::DisplayChanged);
        }
        let left = self.x1.min(self.x2).clamp(0.0, width);
        let right = self.x1.max(self.x2).clamp(0.0, width);
        let top = self.y1.min(self.y2).clamp(0.0, height);
        let bottom = self.y1.max(self.y2).clamp(0.0, height);
        Ok((right > left && bottom > top).then_some(LogicalRect {
            left,
            top,
            right,
            bottom,
        }))
    }
}
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SelectionOutcome {
    Decoded { response: DecodeResponse },
    Cancelled,
}

fn access<T>(
    window: &WebviewWindow,
    f: impl FnOnce(&mut Active) -> Result<T, CaptureError>,
) -> Result<T, CaptureError> {
    let state = window.state::<SelectionState>();
    let mut slot = state.0.lock().map_err(|_| CaptureError::CaptureFailed)?;
    let active = slot
        .as_mut()
        .filter(|active| active.label == window.label())
        .ok_or(CaptureError::WindowFailed)?;
    f(active)
}
#[tauri::command]
pub fn selection_image(window: WebviewWindow) -> Result<tauri::ipc::Response, CaptureFailure> {
    access(&window, |active| {
        Ok(tauri::ipc::Response::new(active.png.clone()))
    })
    .map_err(Into::into)
}
#[tauri::command]
pub fn selection_ready(window: WebviewWindow) -> Result<(), CaptureFailure> {
    access(&window, |active| {
        window
            .set_position(PhysicalPosition::new(active.display.x, active.display.y))
            .map_err(|_| CaptureError::WindowFailed)?;
        window
            .set_size(PhysicalSize::new(
                active.display.width,
                active.display.height,
            ))
            .map_err(|_| CaptureError::WindowFailed)?;
        let size = window
            .inner_size()
            .map_err(|_| CaptureError::WindowFailed)?;
        if size.width != active.display.width || size.height != active.display.height {
            return Err(CaptureError::DisplayChanged);
        }
        window.show().map_err(|_| CaptureError::WindowFailed)?;
        window.set_focus().map_err(|_| CaptureError::WindowFailed)?;
        let scale = window
            .scale_factor()
            .map_err(|_| CaptureError::WindowFailed)?;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(CaptureError::DisplayChanged);
        }
        active.scale = Some(scale);
        active.ready = true;
        Ok(())
    })
    .map_err(Into::into)
}
#[tauri::command]
pub fn selection_commit(
    window: WebviewWindow,
    selection: SelectionInput,
) -> Result<bool, CaptureFailure> {
    access(&window, |active| {
        if !active.ready {
            return Err(CaptureError::WindowFailed);
        }
        let scale = window
            .scale_factor()
            .map_err(|_| CaptureError::WindowFailed)?;
        let size = window
            .inner_size()
            .map_err(|_| CaptureError::WindowFailed)?;
        let position = window
            .inner_position()
            .map_err(|_| CaptureError::WindowFailed)?;
        if active.scale != Some(scale)
            || size.width != active.display.width
            || size.height != active.display.height
            || position.x != active.display.x
            || position.y != active.display.y
        {
            return Err(CaptureError::DisplayChanged);
        }
        let rect = selection.normalize(
            f64::from(active.display.width) / scale,
            f64::from(active.display.height) / scale,
        )?;
        if let Some(rect) = rect {
            let rect = crop::from_overlay(
                Point {
                    x: rect.left,
                    y: rect.top,
                },
                Point {
                    x: rect.right,
                    y: rect.bottom,
                },
                scale,
                active.display.width,
                active.display.height,
            )
            .map_err(|_| CaptureError::DisplayChanged)?;
            active
                .sender
                .try_send(Action::Select(rect))
                .map_err(|_| CaptureError::Busy)?;
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .map_err(Into::into)
}
#[tauri::command]
pub fn selection_cancel(window: WebviewWindow) -> Result<(), CaptureFailure> {
    access(&window, |active| {
        active.cancelled.store(true, Ordering::Release);
        let _ = active.sender.try_send(Action::Cancel);
        Ok(())
    })
    .map_err(Into::into)
}
/// Called for native close events too; never waits on a worker or holds the mutex across UI calls.
pub fn cancel_for_window(app: &tauri::AppHandle, label: &str) {
    if let Ok(slot) = app.state::<SelectionState>().0.lock() {
        if let Some(active) = slot.as_ref() {
            if label == "main" || label == active.label {
                active.cancelled.store(true, Ordering::Release);
                let _ = active.sender.try_send(Action::Cancel);
            }
        }
    }
}
struct Cleanup {
    app: tauri::AppHandle,
    label: String,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Some(window) = self.app.get_webview_window(&self.label) {
            let _ = window.destroy();
        }
        if let Ok(mut slot) = self.app.state::<SelectionState>().0.lock() {
            *slot = None;
        }
    }
}
#[cfg(target_os = "windows")]
fn select(app: tauri::AppHandle, generation: u64) -> Result<SelectionOutcome, CaptureError> {
    capture::wait_for_repaint()?;
    let frame = capture::capture_primary()?;
    let mut png = std::io::Cursor::new(Vec::new());
    // Borrow the frame for encoding; keep original native pixels until session cleanup.
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            frame.pixels.as_raw(),
            frame.display.width,
            frame.display.height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|_| CaptureError::CaptureFailed)?;
    let label = format!("selection-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let (sender, receiver) = mpsc::sync_channel(1);
    let cancelled = Arc::new(AtomicBool::new(false));
    *app.state::<SelectionState>()
        .0
        .lock()
        .map_err(|_| CaptureError::CaptureFailed)? = Some(Active {
        label: label.clone(),
        png: png.into_inner(),
        display: frame.display.clone(),
        sender,
        ready: false,
        cancelled: cancelled.clone(),
        scale: None,
        review: None,
    });
    let _cleanup = Cleanup {
        app: app.clone(),
        label: label.clone(),
    };
    let _overlay = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::App("index.html?selection=1".into()),
    )
    .title("ScreenQR region selection")
    .decorations(false)
    .resizable(false)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .focused(false)
    .build()
    .map_err(|_| CaptureError::WindowFailed)?;
    let started = Instant::now();
    let mut checked = Instant::now();
    loop {
        // Poll only during this explicit scan; no registered/global shortcut or background capture.
        // SAFETY: virtual key constant, no pointers or owned resources.
        if cancelled.load(Ordering::Acquire) || escape_pressed() {
            return Ok(SelectionOutcome::Cancelled);
        }
        if checked.elapsed() >= Duration::from_millis(500) {
            if capture::desktop_metrics()? != frame.topology {
                return Err(CaptureError::DisplayChanged);
            }
            checked = Instant::now();
        }
        let ready = app
            .state::<SelectionState>()
            .0
            .lock()
            .map_err(|_| CaptureError::CaptureFailed)?
            .as_ref()
            .is_some_and(|active| active.ready);
        if !ready && started.elapsed() > Duration::from_secs(15) {
            return Err(CaptureError::WindowFailed);
        }
        if app.get_webview_window("main").is_none() {
            return Ok(SelectionOutcome::Cancelled);
        }
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(Action::Cancel) => return Ok(SelectionOutcome::Cancelled),
            Ok(Action::Select(rectangle)) => {
                if capture::desktop_metrics()? != frame.topology {
                    return Err(CaptureError::DisplayChanged);
                }
                // Bound decoder allocation before copying the crop.
                let dimensions = decoder::validate_dimensions(rectangle.width, rectangle.height);
                let cropped = if dimensions.is_ok() {
                    Some(
                        crop::crop_rgba(&frame.pixels, rectangle)
                            .map_err(|_| CaptureError::DisplayChanged)?,
                    )
                } else {
                    None
                };
                let topology = frame.topology.clone();
                drop(frame);
                if let Ok(mut slot) = app.state::<SelectionState>().0.lock() {
                    if let Some(active) = slot.as_mut() {
                        active.ready = false;
                        active.png = Vec::new();
                    }
                }
                let mut display_changed = false;
                let mut response = std::thread::scope(|scope| {
                    let worker = scope.spawn(move || match cropped {
                        Some(image) => decoder::decode_rgba(image),
                        None => Err(dimensions
                            .err()
                            .unwrap_or_else(|| DecodeError::new(ErrorCode::InternalError))),
                    });
                    while !worker.is_finished() {
                        if escape_pressed() || app.get_webview_window("main").is_none() {
                            cancelled.store(true, Ordering::Release);
                        }
                        if checked.elapsed() >= Duration::from_millis(500) {
                            display_changed |= capture::desktop_metrics()
                                .map_or(true, |current| current != topology);
                            checked = Instant::now();
                        }
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    worker
                        .join()
                        .unwrap_or_else(|_| Err(DecodeError::new(ErrorCode::InternalError)))
                });
                if cancelled.load(Ordering::Acquire) || escape_pressed() {
                    return Ok(SelectionOutcome::Cancelled);
                }
                if display_changed || capture::desktop_metrics()? != topology {
                    return Err(CaptureError::DisplayChanged);
                }
                if let Ok(batch) = &mut response {
                    crate::actions::register(batch, Some(&label), generation);
                    let scale = app
                        .state::<SelectionState>()
                        .0
                        .lock()
                        .map_err(|_| CaptureError::CaptureFailed)?
                        .as_ref()
                        .and_then(|active| active.scale)
                        .ok_or(CaptureError::WindowFailed)?;
                    let anchors = batch
                        .results
                        .iter()
                        .map(|result| {
                            result
                                .bounds
                                .and_then(|bounds| review_anchor(bounds, rectangle, scale))
                                .ok_or(CaptureError::DisplayChanged)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    {
                        let state = app.state::<SelectionState>();
                        let mut slot = state.0.lock().map_err(|_| CaptureError::CaptureFailed)?;
                        let active = slot.as_mut().ok_or(CaptureError::WindowFailed)?;
                        active.review = Some(OverlayReview {
                            batch: batch.clone(),
                            anchors,
                        });
                    }
                    // Review dismissal retains this already-decoded batch for the main list.
                    loop {
                        let review_focused = app
                            .get_webview_window(&label)
                            .is_some_and(|window| window.is_focused().unwrap_or(false));
                        if cancelled.load(Ordering::Acquire) || (review_focused && escape_pressed())
                        {
                            break;
                        }
                        if app.get_webview_window("main").is_none() {
                            return Ok(SelectionOutcome::Cancelled);
                        }
                        if checked.elapsed() >= Duration::from_millis(500) {
                            if capture::desktop_metrics()? != topology {
                                return Err(CaptureError::DisplayChanged);
                            }
                            let window = app
                                .get_webview_window(&label)
                                .ok_or(CaptureError::WindowFailed)?;
                            if window
                                .scale_factor()
                                .map_err(|_| CaptureError::WindowFailed)?
                                != scale
                            {
                                return Err(CaptureError::DisplayChanged);
                            }
                            checked = Instant::now();
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
                return Ok(SelectionOutcome::Decoded {
                    response: response.into(),
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => return Ok(SelectionOutcome::Cancelled),
        }
    }
}
#[cfg(target_os = "windows")]
use image::ImageEncoder;
#[cfg(not(target_os = "windows"))]
fn select(_: tauri::AppHandle, _generation: u64) -> Result<SelectionOutcome, CaptureError> {
    Err(CaptureError::UnsupportedPlatform)
}
#[tauri::command]
pub async fn start_selection(window: WebviewWindow) -> Result<SelectionOutcome, CaptureFailure> {
    if window.label() != "main" {
        return Err(CaptureError::WindowFailed.into());
    }
    let permit = Permit::acquire()?;
    let generation = crate::actions::clear();
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        let result = with_hidden_surface(&MainSurface(window), || select(app, generation));
        if !matches!(&result, Ok(SelectionOutcome::Decoded { .. })) {
            crate::actions::clear_generation(generation);
        }
        result
    })
    .await
    .map_err(|_| CaptureFailure::from(CaptureError::CaptureFailed))?
    .map_err(Into::into)
}

#[cfg(target_os = "windows")]
fn escape_pressed() -> bool {
    // SAFETY: virtual-key query only; no pointers or registered global shortcut.
    unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(0x1B) < 0 }
}
#[tauri::command]
pub fn cancel_scan(window: WebviewWindow) -> Result<(), CaptureFailure> {
    if window.label() != "main" {
        return Err(CaptureError::WindowFailed.into());
    }
    cancel_for_window(window.app_handle(), "main");
    Ok(())
}

#[derive(Clone, Serialize)]
pub struct OverlayReview {
    batch: DecodeBatch,
    anchors: Vec<LogicalRect>,
}
fn review_anchor(bounds: QrBounds, crop: PixelRect, scale: f64) -> Option<LogicalRect> {
    if !scale.is_finite()
        || scale <= 0.0
        || bounds.width == 0
        || bounds.height == 0
        || bounds.x.checked_add(bounds.width)? > crop.width
        || bounds.y.checked_add(bounds.height)? > crop.height
    {
        return None;
    }
    let left = f64::from(crop.x) + f64::from(bounds.x);
    let top = f64::from(crop.y) + f64::from(bounds.y);
    Some(LogicalRect {
        left: left / scale,
        top: top / scale,
        right: (left + f64::from(bounds.width)) / scale,
        bottom: (top + f64::from(bounds.height)) / scale,
    })
}
#[tauri::command]
pub fn selection_review(window: WebviewWindow) -> Result<Option<OverlayReview>, CaptureFailure> {
    access(&window, |active| Ok(active.review.clone())).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anchors_use_crop_offset_and_one_inverse_dpi_transform() {
        let bounds = QrBounds {
            x: 30,
            y: 60,
            width: 90,
            height: 120,
        };
        let crop = PixelRect {
            x: 120,
            y: 90,
            width: 500,
            height: 500,
        };
        assert_eq!(
            review_anchor(bounds, crop, 1.5),
            Some(LogicalRect {
                left: 100.0,
                top: 100.0,
                right: 160.0,
                bottom: 180.0
            })
        );
        assert!(review_anchor(bounds, crop, 0.0).is_none());
        assert!(review_anchor(QrBounds { x: 499, ..bounds }, crop, 1.5).is_none());
    }
    #[test]
    fn all_drag_directions_produce_same_logical_rectangle() {
        for (x1, x2) in [(10.0, 90.0), (90.0, 10.0)] {
            for (y1, y2) in [(20.0, 80.0), (80.0, 20.0)] {
                let input = SelectionInput {
                    x1,
                    y1,
                    x2,
                    y2,
                    viewport_width: 100.0,
                    viewport_height: 100.0,
                };
                assert_eq!(
                    input.normalize(100.0, 100.0),
                    Ok(Some(LogicalRect {
                        left: 10.0,
                        top: 20.0,
                        right: 90.0,
                        bottom: 80.0
                    }))
                );
            }
        }
    }
    #[test]
    fn empty_clamped_and_invalid_selections_are_controlled() {
        let mut input = SelectionInput {
            x1: -10.0,
            y1: -10.0,
            x2: 110.0,
            y2: 110.0,
            viewport_width: 100.0,
            viewport_height: 100.0,
        };
        assert_eq!(
            input.normalize(100.0, 100.0),
            Ok(Some(LogicalRect {
                left: 0.0,
                top: 0.0,
                right: 100.0,
                bottom: 100.0
            }))
        );
        input.x2 = -10.0;
        assert_eq!(input.normalize(100.0, 100.0), Ok(None));
        input.x1 = f64::NAN;
        assert_eq!(
            input.normalize(100.0, 100.0),
            Err(CaptureError::DisplayChanged)
        );
        input.x1 = 0.0;
        assert_eq!(
            input.normalize(200.0, 100.0),
            Err(CaptureError::DisplayChanged)
        );
    }
}
