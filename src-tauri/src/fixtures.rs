//! Fixed synthetic test assets. No caller-supplied paths or filesystem access.
use crate::decoder::types::{DecodeError, ErrorCode};

pub fn image(id: &str) -> Result<&'static [u8], DecodeError> {
    if id.len() > 16 {
        return Err(DecodeError::new(ErrorCode::InvalidFixture));
    }
    match id {
        "url" => Ok(include_bytes!("../tests/fixtures/url.png")),
        "thai" => Ok(include_bytes!("../tests/fixtures/thai.png")),
        "emoji" => Ok(include_bytes!("../tests/fixtures/emoji.png")),
        "multiline" => Ok(include_bytes!("../tests/fixtures/multiline.png")),
        "html" => Ok(include_bytes!("../tests/fixtures/html.png")),
        "blank" => Ok(include_bytes!("../tests/fixtures/blank.png")),
        "damaged" => Ok(include_bytes!("../tests/fixtures/damaged.png")),
        "multiple" => Ok(include_bytes!("../tests/fixtures/multiple.png")),
        "binary" => Ok(include_bytes!("../tests/fixtures/binary.png")),
        "invalid" => Ok(b"not a PNG image"),
        _ => Err(DecodeError::new(ErrorCode::InvalidFixture)),
    }
}
