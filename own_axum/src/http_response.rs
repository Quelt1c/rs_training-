use super::content_type::ContentType;
use super::status_code::StatusCode;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub type StreamWriteFn = Box<
    dyn for<'a> FnOnce(
            &'a mut (dyn tokio::io::AsyncWrite + Unpin + Send),
        ) -> Pin<Box<dyn Future<Output = std::io::Result<()>> + Send + 'a>>
        + Send,
>;

pub(crate) struct StreamBody {
    pub(crate) content_length: u64,
    pub(crate) write: StreamWriteFn,
}

pub(crate) enum ResponseBody {
    Bytes(Vec<u8>),
    Stream(StreamBody),
}

pub struct HttpResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: ResponseBody,
}

impl HttpResponse {
    /// Attach (or overwrite) one header on the response, e.g.
    /// `.with_header("set-cookie", "session_id=...; HttpOnly")`.
    /// Note: only one value per header name is kept — setting multiple
    /// cookies in a single response isn't supported.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_lowercase(), value.to_string());
        self
    }

    pub fn new(status: StatusCode, body: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            ContentType::TextPlain.as_str().to_string(),
        );

        Self {
            status,
            headers,
            body: ResponseBody::Bytes(body.as_bytes().to_vec()),
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
            body: ResponseBody::Bytes(body.into_bytes()),
        }
    }

    /// A response whose body is produced by streaming, e.g. reading a file
    /// or generating a zip archive chunk-by-chunk straight onto the wire.
    /// `content_length` must be known up front (computed from file sizes,
    /// never by reading file content) since the response is sent without
    /// chunked transfer-encoding.
    pub fn stream(
        status: StatusCode,
        content_type: ContentType,
        filename: Option<&str>,
        content_length: u64,
        write: StreamWriteFn,
    ) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            content_type.as_str().to_string(),
        );
        if let Some(name) = filename {
            headers.insert(
                "content-disposition".to_string(),
                format!("attachment; filename=\"{name}\""),
            );
        }

        Self {
            status,
            headers,
            body: ResponseBody::Stream(StreamBody {
                content_length,
                write,
            }),
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
