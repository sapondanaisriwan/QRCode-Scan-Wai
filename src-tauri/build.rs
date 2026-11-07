fn main() {
    if let Err(error) = tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "copy_result",
            "open_result",
            "decode_fixture",
            "capture_preview",
            "start_selection",
            "selection_image",
            "selection_ready",
            "selection_commit",
            "selection_cancel",
            "selection_review",
            "cancel_scan",
        ]),
    )) {
        eprintln!("Tauri build configuration failed: {error}");
        std::process::exit(1);
    }
}
