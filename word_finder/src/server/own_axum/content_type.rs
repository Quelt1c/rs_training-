#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    TextPlain,
    ApplicationJson,
    ApplicationOctetStream,
    ApplicationFormUrlEncoded,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::TextPlain => "text/plain; charset=utf-8",
            ContentType::ApplicationJson => "application/json",
            ContentType::ApplicationOctetStream => "application/octet-stream",
            ContentType::ApplicationFormUrlEncoded => "application/x-www-form-urlencoded",
        }
    }

    pub fn from_header(value: &str) -> Option<Self> {
        let base = value
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();

        match base.as_str() {
            "text/plain" => Some(ContentType::TextPlain),
            "application/json" => Some(ContentType::ApplicationJson),
            "application/octet-stream" => Some(ContentType::ApplicationOctetStream),
            "application/x-www-form-urlencoded" => Some(ContentType::ApplicationFormUrlEncoded),
            _ => None,
        }
    }
}
