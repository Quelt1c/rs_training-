use super::content_type::ContentType;
use super::status_code::StatusCode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: StatusCode, body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            ContentType::TextPlain.as_str().to_string(),
        );

        Self {
            status,
            headers,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn json(status: StatusCode, body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            ContentType::ApplicationJson.as_str().to_string(),
        );

        Self {
            status,
            headers,
            body: body.into_bytes(),
        }
    }

    pub fn file(filename: &str, content: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            ContentType::ApplicationOctetStream.as_str().to_string(),
        );
        headers.insert(
            "content-disposition".to_string(),
            format!("attachment; filename=\"{filename}\""),
        );

        Self {
            status: StatusCode::OK,
            headers,
            body: content,
        }
    }
}

impl From<(StatusCode, String)> for HttpResponse {
    fn from(val: (StatusCode, String)) -> Self {
        HttpResponse::new(val.0, &val.1)
    }
}

impl From<(StatusCode, &'static str)> for HttpResponse {
    fn from(val: (StatusCode, &'static str)) -> Self {
        HttpResponse::new(val.0, val.1)
    }
}
