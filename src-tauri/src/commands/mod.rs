pub mod capture;
use crate::{
    decoder::{
        self,
        types::{DecodeError, DecodeResponse, ErrorCode},
    },
    fixtures,
};
use std::sync::atomic::{AtomicBool, Ordering};

static DECODING: AtomicBool = AtomicBool::new(false);
struct DecodePermit;
impl Drop for DecodePermit {
    fn drop(&mut self) {
        DECODING.store(false, Ordering::Release);
    }
}

#[tauri::command]
pub async fn decode_fixture(fixture: String) -> DecodeResponse {
    let bytes = match fixtures::image(&fixture) {
        Ok(bytes) => bytes,
        Err(error) => return DecodeResponse::Error { error },
    };
    if DECODING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(DecodeError::new(ErrorCode::Busy)).into();
    }
    let permit = DecodePermit;
    match tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        decoder::decode_image(bytes)
    })
    .await
    {
        Ok(result) => result.into(),
        Err(_) => Err(DecodeError::new(ErrorCode::InternalError)).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_returns_structured_results_and_releases_busy_guard() {
        tauri::async_runtime::block_on(async {
            let response = decode_fixture("thai".into()).await;
            let json = serde_json::to_value(response).unwrap();
            assert_eq!(json["status"], "success");
            assert_eq!(
                json["result"]["results"][0]["content"],
                "สวัสดี ชาวโลก เลขอ้างอิง ๑๒๓"
            );
            let error = decode_fixture("../secret.png".into()).await;
            assert_eq!(
                serde_json::to_value(error).unwrap()["error"]["code"],
                "invalid_fixture"
            );
            DECODING.store(true, Ordering::Release);
            let busy = decode_fixture("url".into()).await;
            assert_eq!(serde_json::to_value(busy).unwrap()["error"]["code"], "busy");
            drop(DecodePermit);
            let retry = decode_fixture("blank".into()).await;
            assert_eq!(
                serde_json::to_value(retry).unwrap()["error"]["code"],
                "no_qr_detected"
            );
            assert!(!DECODING.load(Ordering::Acquire));
        });
    }
}
