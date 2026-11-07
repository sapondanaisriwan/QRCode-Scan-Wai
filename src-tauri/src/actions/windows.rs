//! Narrow Windows wrappers; no shell commands, clipboard reads or persistent data.
use super::ActionError;
use ::windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        },
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    },
};
const CF_UNICODETEXT: u32 = 13;
struct Memory(Option<HGLOBAL>);
impl Drop for Memory {
    fn drop(&mut self) {
        if let Some(handle) = self.0 {
            unsafe {
                let _ = GlobalFree(Some(handle));
            }
        }
    }
}
struct Clipboard;
impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
struct Com;
impl Drop for Com {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
pub fn copy(window: &tauri::WebviewWindow, text: &[u16]) -> Result<(), ActionError> {
    let hwnd = HWND(window.hwnd().map_err(|_| ActionError::ClipboardFailed)?.0);
    // SAFETY: allocate enough bytes for the validated terminated UTF-16 slice; guard frees
    // memory until SetClipboardData successfully transfers ownership to Windows.
    unsafe {
        let handle = GlobalAlloc(GMEM_MOVEABLE, std::mem::size_of_val(text))
            .map_err(|_| ActionError::ClipboardFailed)?;
        let mut memory = Memory(Some(handle));
        let target = GlobalLock(handle).cast::<u16>();
        if target.is_null() {
            return Err(ActionError::ClipboardFailed);
        }
        std::ptr::copy_nonoverlapping(text.as_ptr(), target, text.len());
        let _ = GlobalUnlock(handle);
        OpenClipboard(Some(hwnd)).map_err(|_| ActionError::ClipboardFailed)?;
        let _clipboard = Clipboard;
        EmptyClipboard().map_err(|_| ActionError::ClipboardFailed)?;
        SetClipboardData(CF_UNICODETEXT, Some(HANDLE(handle.0)))
            .map_err(|_| ActionError::ClipboardFailed)?;
        memory.0 = None;
    }
    Ok(())
}
pub fn open(window: &tauri::WebviewWindow, destination: &str) -> Result<(), ActionError> {
    let hwnd = window.hwnd().map_err(|_| ActionError::OpenFailed)?.0 as usize;
    let wide: Vec<u16> = destination
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // A fresh thread guarantees a known STA apartment for Windows shell extensions.
    std::thread::Builder::new()
        .spawn(move || {
            // SAFETY: all strings are live, terminated buffers; destination was revalidated
            // as HTTP(S). No executable, parameters, directory or arbitrary verb is supplied.
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                    .ok()
                    .map_err(|_| ActionError::OpenFailed)?;
                let _com = Com;
                let result = ShellExecuteW(
                    Some(HWND(hwnd as *mut _)),
                    w!("open"),
                    PCWSTR(wide.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOWNORMAL,
                );
                if result.0 as isize <= 32 {
                    Err(ActionError::OpenFailed)
                } else {
                    Ok(())
                }
            }
        })
        .map_err(|_| ActionError::OpenFailed)?
        .join()
        .unwrap_or(Err(ActionError::OpenFailed))
}
