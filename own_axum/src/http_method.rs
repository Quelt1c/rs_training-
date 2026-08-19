#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("GET") {
            Some(HttpMethod::GET)
        } else if s.eq_ignore_ascii_case("PUT") {
            Some(HttpMethod::PUT)
        } else if s.eq_ignore_ascii_case("POST") {
            Some(HttpMethod::POST)
        } else {
            None
        }
    }
}
