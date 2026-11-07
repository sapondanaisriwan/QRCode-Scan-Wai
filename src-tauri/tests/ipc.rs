use serde_json::{json, Value};

#[test]
fn decoder_capability_is_local_and_main_only() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    assert_eq!(capability["windows"], json!(["main"]));
    assert_eq!(
        capability["permissions"],
        json!([
            "allow-decode-fixture",
            "allow-capture-preview",
            "allow-start-selection",
            "allow-cancel-scan",
            "allow-copy-result",
            "allow-open-result"
        ])
    );
    assert!(capability.get("remote").is_none());
    let build = include_str!("../build.rs");
    for command in [
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
    ] {
        assert!(build.contains(command));
    }
}

#[test]
fn overlay_has_no_capture_or_fixture_permission() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/selection.json")).unwrap();
    assert_eq!(capability["windows"], json!(["selection-*"]));
    assert_eq!(
        capability["permissions"],
        json!([
            "allow-selection-image",
            "allow-selection-ready",
            "allow-selection-commit",
            "allow-selection-cancel",
            "allow-selection-review",
            "allow-copy-result",
            "allow-open-result"
        ])
    );
    assert!(capability.get("remote").is_none());
}
