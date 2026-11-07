use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidImage,
    ImageTooLarge,
    NoQrDetected,
    MalformedQr,
    UnsupportedContent,
    InvalidFixture,
    Busy,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecodeError {
    pub code: ErrorCode,
    pub message: &'static str,
}

impl DecodeError {
    pub fn new(code: ErrorCode) -> Self {
        let message = match code {
            ErrorCode::InvalidImage => "The image is not a valid PNG. Try another image.",
            ErrorCode::ImageTooLarge => "The image exceeds the decoder size limit.",
            ErrorCode::NoQrDetected => {
                "No QR code detected. Retry with the full code and its white border."
            }
            ErrorCode::MalformedQr => "The QR code could not be read. Retry with a clearer image.",
            ErrorCode::UnsupportedContent => "This QR code does not contain supported UTF-8 text.",
            ErrorCode::InvalidFixture => "Choose one of the bundled test images.",
            ErrorCode::Busy => "A decode is already running. Try again when it finishes.",
            ErrorCode::InternalError => "Decoding failed. Please retry.",
        };
        Self { code, message }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Url,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    /// Axis-aligned bounds in the decoded image's physical pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<QrBounds>,
    pub content: String,
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub launch_destination: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeBatch {
    pub results: Vec<ScanResult>,
    pub unreadable_count: usize,
    pub unsupported_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DecodeResponse {
    Success { result: DecodeBatch },
    Error { error: DecodeError },
}

impl From<Result<DecodeBatch, DecodeError>> for DecodeResponse {
    fn from(value: Result<DecodeBatch, DecodeError>) -> Self {
        match value {
            Ok(result) => Self::Success { result },
            Err(error) => Self::Error { error },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct QrBounds {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
