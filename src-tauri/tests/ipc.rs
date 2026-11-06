use serde_json::{json, Value};

#[test]
fn decoder_capability_is_local_and_main_only() {
    let capability: Value =
        serde_json::from_str(include_str!("../capabilities/main.json")).unwrap();
    assert_eq!(capability["windows"], json!(["main"]));
    assert_eq!(
        capability["permissions"],
        json!(["allow-decode-fixture", "allow-capture-preview"])
    );
    assert!(capability.get("remote").is_none());
    let build = include_str!("../build.rs");
    assert!(build.contains(".commands(&[\"decode_fixture\", \"capture_preview\"])"));
}
