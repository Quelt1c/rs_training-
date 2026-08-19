#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    TextPlain,
    ApplicationJson,
    ApplicationOctetStream,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::TextPlain => "text/plain; charset=utf-8",
            ContentType::ApplicationJson => "application/json",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}
