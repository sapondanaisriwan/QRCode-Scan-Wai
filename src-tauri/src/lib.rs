use tauri::Manager;

pub mod actions;
pub mod capture;
pub mod commands;
pub mod decoder;
pub mod fixtures;

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(commands::selection::SelectionState::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                commands::selection::cancel_for_window(window.app_handle(), window.label());
                if window.label().starts_with("selection-") {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            actions::copy_result,
            actions::open_result,
            commands::decode_fixture,
            commands::capture::capture_preview,
            commands::selection::start_selection,
            commands::selection::selection_image,
            commands::selection::selection_ready,
            commands::selection::selection_commit,
            commands::selection::selection_cancel,
            commands::selection::selection_review,
            commands::selection::cancel_scan
        ])
        .run(tauri::generate_context!())
}
