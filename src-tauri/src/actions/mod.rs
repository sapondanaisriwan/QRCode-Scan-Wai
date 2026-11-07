//! Result-scoped actions. IPC never accepts arbitrary clipboard text or a URL to launch.
use crate::decoder::{
    content::validated_destination,
    types::{DecodeBatch, ScanResult},
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Mutex,
};
#[cfg(target_os = "windows")]
mod windows;
struct Registry {
    generation: u64,
    grants: Vec<Grant>,
}
static STORE: Mutex<Registry> = Mutex::new(Registry {
    generation: 0,
    grants: Vec::new(),
});
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static BUSY: AtomicBool = AtomicBool::new(false);
#[derive(Clone)]
struct Grant {
    id: String,
    overlay: Option<String>,
    result: ScanResult,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionError {
    ExpiredResult,
    InvalidDestination,
    ClipboardFailed,
    UnsupportedText,
    OpenFailed,
    Busy,
    UnsupportedPlatform,
}
#[derive(Serialize)]
pub struct ActionFailure {
    code: ActionError,
    message: &'static str,
}
impl From<ActionError> for ActionFailure {
    fn from(code: ActionError) -> Self {
        let message = match code {
            ActionError::ExpiredResult => "This result has expired. Scan or decode again.",
            ActionError::InvalidDestination => {
                "This result is not an eligible HTTP or HTTPS destination."
            }
            ActionError::ClipboardFailed => {
                "Cannot copy now. The clipboard may be busy. Retry Copy."
            }
            ActionError::UnsupportedText => {
                "This text cannot be represented on the Windows text clipboard without truncation."
            }
            ActionError::OpenFailed => {
                "Cannot open the default browser. You can still copy this result."
            }
            ActionError::Busy => "An action is already running. Please wait and retry.",
            ActionError::UnsupportedPlatform => "This action requires Windows.",
        };
        Self { code, message }
    }
}
impl Registry {
    fn reset(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.grants.clear();
        self.generation
    }
    fn register(&mut self, batch: &mut DecodeBatch, overlay: Option<&str>, generation: u64) {
        if self.generation != generation {
            return;
        }
        for result in &mut batch.results {
            let id = format!("result-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
            result.action_id = Some(id.clone());
            self.grants.push(Grant {
                id,
                overlay: overlay.map(str::to_owned),
                result: result.clone(),
            });
        }
    }
}
pub fn clear() -> u64 {
    STORE.lock().map(|mut state| state.reset()).unwrap_or(0)
}
pub fn clear_generation(generation: u64) {
    if let Ok(mut state) = STORE.lock() {
        if state.generation == generation {
            state.reset();
        }
    }
}
pub fn register(batch: &mut DecodeBatch, overlay: Option<&str>, generation: u64) {
    if let Ok(mut state) = STORE.lock() {
        state.register(batch, overlay, generation);
    }
}
fn lookup(entries: &[Grant], label: &str, id: &str) -> Result<ScanResult, ActionError> {
    if id.len() > 64 {
        return Err(ActionError::ExpiredResult);
    }
    entries
        .iter()
        .find(|entry| {
            entry.id == id && (label == "main" || entry.overlay.as_deref() == Some(label))
        })
        .map(|entry| entry.result.clone())
        .ok_or(ActionError::ExpiredResult)
}
fn resolve(label: &str, id: &str) -> Result<ScanResult, ActionError> {
    lookup(
        &STORE.lock().map_err(|_| ActionError::ExpiredResult)?.grants,
        label,
        id,
    )
}
fn destination(result: &ScanResult, displayed: &str) -> Result<String, ActionError> {
    if displayed.len() > 65536 {
        return Err(ActionError::InvalidDestination);
    }
    let destination =
        validated_destination(&result.content).ok_or(ActionError::InvalidDestination)?;
    if result.launch_destination.as_deref() != Some(destination.as_str())
        || displayed != destination
    {
        return Err(ActionError::InvalidDestination);
    }
    Ok(destination)
}
fn clipboard_text(content: &str) -> Result<Vec<u16>, ActionError> {
    if content.len() > 65536 || content.contains('\0') {
        return Err(ActionError::UnsupportedText);
    }
    Ok(content.encode_utf16().chain(std::iter::once(0)).collect())
}
struct Permit;
impl Permit {
    fn acquire() -> Result<Self, ActionError> {
        BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| ActionError::Busy)
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::Release);
    }
}
#[tauri::command]
pub async fn copy_result(
    window: tauri::WebviewWindow,
    action_id: String,
) -> Result<(), ActionFailure> {
    let permit = Permit::acquire()?;
    let result = resolve(window.label(), &action_id)?;
    let text = clipboard_text(&result.content)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        #[cfg(target_os = "windows")]
        {
            windows::copy(&window, &text)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (window, text);
            Err(ActionError::UnsupportedPlatform)
        }
    })
    .await
    .map_err(|_| ActionFailure::from(ActionError::ClipboardFailed))?
    .map_err(Into::into)
}
#[tauri::command]
pub async fn open_result(
    window: tauri::WebviewWindow,
    action_id: String,
    displayed_destination: String,
) -> Result<(), ActionFailure> {
    let permit = Permit::acquire()?;
    let result = resolve(window.label(), &action_id)?;
    let destination = destination(&result, &displayed_destination)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        #[cfg(target_os = "windows")]
        {
            let overlay = window.label().starts_with("selection-");
            // Let the browser appear above review; keep review reachable from the taskbar.
            if overlay {
                window
                    .set_skip_taskbar(false)
                    .map_err(|_| ActionError::OpenFailed)?;
                window
                    .set_always_on_top(false)
                    .map_err(|_| ActionError::OpenFailed)?;
            }
            let outcome = windows::open(&window, &destination);
            if overlay && outcome.is_err() {
                let _ = window.set_always_on_top(true);
                let _ = window.set_skip_taskbar(true);
            }
            outcome
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (window, destination);
            Err(ActionError::UnsupportedPlatform)
        }
    })
    .await
    .map_err(|_| ActionFailure::from(ActionError::OpenFailed))?
    .map_err(Into::into)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::content::classify;
    #[test]
    fn replacement_invalidates_ids_and_late_decodes_cannot_restore_them() {
        let mut registry = Registry {
            generation: 0,
            grants: Vec::new(),
        };
        let old = registry.reset();
        let mut batch = DecodeBatch {
            results: vec![classify("hello".into())],
            unreadable_count: 0,
            unsupported_count: 0,
        };
        registry.register(&mut batch, Some("selection-1"), old);
        let id = batch.results[0].action_id.clone().unwrap();
        let current = registry.reset();
        assert!(lookup(&registry.grants, "main", &id).is_err());
        registry.register(&mut batch, None, old);
        assert!(registry.grants.is_empty());
        registry.register(&mut batch, None, current);
        assert_ne!(batch.results[0].action_id.as_deref(), Some(id.as_str()));
    }
    #[test]
    fn action_permit_rejects_duplicates_and_releases_for_retry() {
        let permit = Permit::acquire().unwrap();
        assert!(matches!(Permit::acquire(), Err(ActionError::Busy)));
        drop(permit);
        assert!(Permit::acquire().is_ok());
    }
    #[test]
    fn window_and_result_ownership_are_checked() {
        let entries = vec![Grant {
            id: "result-1".into(),
            overlay: Some("selection-9".into()),
            result: classify("hello".into()),
        }];
        assert!(lookup(&entries, "main", "result-1").is_ok());
        assert!(lookup(&entries, "selection-9", "result-1").is_ok());
        assert_eq!(
            lookup(&entries, "selection-8", "result-1").unwrap_err(),
            ActionError::ExpiredResult
        );
        assert_eq!(
            lookup(&[], "main", "result-1").unwrap_err(),
            ActionError::ExpiredResult
        );
    }
    #[test]
    fn opening_revalidates_original_and_exact_displayed_destination() {
        let result = classify("HTTPS://example.com/path?q=1".into());
        assert_eq!(
            destination(&result, "https://example.com/path?q=1").unwrap(),
            "https://example.com/path?q=1"
        );
        assert_eq!(
            destination(&result, "https://evil.example"),
            Err(ActionError::InvalidDestination)
        );
        for source in [
            "javascript:alert(1)",
            "file:///C:/test",
            "https://user@example.com",
            "https://example.com\\path",
            " https://example.com",
            "https://example.com/%zz",
        ] {
            let mut result = classify(source.into());
            result.launch_destination = Some("https://example.com/".into());
            assert_eq!(
                destination(&result, "https://example.com/"),
                Err(ActionError::InvalidDestination)
            );
        }
    }
    #[test]
    fn clipboard_preserves_unicode_and_whitespace_and_rejects_nul() {
        let text = "  \u{0e44}\u{0e17}\u{0e22} \u{1f600}\r\nsecond\nlast\t ";
        let encoded = clipboard_text(text).unwrap();
        assert_eq!(encoded.last(), Some(&0));
        assert_eq!(
            String::from_utf16(&encoded[..encoded.len() - 1]).unwrap(),
            text
        );
        assert_eq!(clipboard_text("a\0b"), Err(ActionError::UnsupportedText));
    }
}
