pub mod capture;
pub mod commands;
pub mod decoder;
pub mod fixtures;

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::decode_fixture,
            commands::capture::capture_preview
        ])
        .run(tauri::generate_context!())
}
