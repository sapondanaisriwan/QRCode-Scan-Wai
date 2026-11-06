use super::types::{ContentType, ScanResult};
use url::Url;

/// Pure validation: no DNS, previews, browser calls or source-text mutation.
pub fn validated_destination(content: &str) -> Option<String> {
    if content.is_empty()
        || content.trim() != content
        || content
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return None;
    }
    // Require explicit authority syntax before the WHATWG parser can repair it.
    let (scheme, rest) = content.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') || authority.ends_with(':') {
        return None;
    }
    // Reject malformed percent escapes rather than relying on parser recovery.
    let bytes = content.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit())
        {
            return None;
        }
    }
    let parsed = Url::parse(content).ok()?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none_or(str::is_empty)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return None;
    }
    Some(parsed.to_string())
}

pub fn classify(content: String) -> ScanResult {
    let launch_destination = validated_destination(&content);
    let content_type = if launch_destination.is_some() {
        ContentType::Url
    } else {
        ContentType::Text
    };
    ScanResult {
        content,
        content_type,
        launch_destination,
    }
}
