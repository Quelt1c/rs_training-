#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum StatusCode {
    OK = 200,
    BAD_REQUEST = 400,
    UNAUTHORIZED = 401,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWED = 405,
}

impl StatusCode {
    pub(crate) fn as_parts(&self) -> (u16, &'static str) {
        match self {
            StatusCode::OK => (200, "OK"),
            StatusCode::BAD_REQUEST => (400, "Bad Request"),
            StatusCode::UNAUTHORIZED => (401, "Unauthorized"),
            StatusCode::NOT_FOUND => (404, "Not Found"),
            StatusCode::METHOD_NOT_ALLOWED => (405, "Method Not Allowed"),
        }
    }
}
