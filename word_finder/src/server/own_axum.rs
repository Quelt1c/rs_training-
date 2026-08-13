use anyhow::bail;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum StatusCode {
    OK = 200,
    BAD_REQUEST = 400,
    NOT_FOUND = 404,
    METHOD_NOT_ALLOWED = 405,
}

impl StatusCode {
    fn as_parts(&self) -> (u16, &'static str) {
        match self {
            StatusCode::OK => (200, "OK"),
            StatusCode::BAD_REQUEST => (400, "Bad Request"),
            StatusCode::NOT_FOUND => (404, "Not Found"),
            StatusCode::METHOD_NOT_ALLOWED => (405, "Method Not Allowed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    path: String,
    raw_query: String,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(String::as_str)
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    status: StatusCode,
    body: String,
    content_type: String,
}

impl HttpResponse {
    pub fn new(status: StatusCode, body: &str) -> Self {
        Self {
            status,
            body: body.to_string(),
            content_type: "text/plain; charset=utf-8".to_string(),
        }
    }

    pub fn json(status: StatusCode, body: String) -> Self {
        Self {
            status,
            body,
            content_type: "application/json".to_string(),
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

pub struct Query<T>(pub T);

impl<T: DeserializeOwned> Query<T> {
    fn extract(req: &HttpRequest) -> Result<Self, HttpResponse> {
        serde_urlencoded::from_str::<T>(&req.raw_query)
            .map(Query)
            .map_err(|e| {
                HttpResponse::new(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid query parameters: {e}"),
                )
            })
    }
}

type BoxFuture = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;
type BoxedHandler = Box<dyn Fn(HttpRequest) -> BoxFuture + Send + Sync>;

#[derive(serde::Deserialize)]
pub struct InfoParams {
    pub format: Option<String>,
}

pub async fn info_handler(_req: HttpRequest, query: Query<InfoParams>, _state: ()) -> HttpResponse {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    match query.0.format.as_deref() {
        Some("json") => {
            let body = format!(r#"{{"name": "{name}", "version": "{version}"}}"#);
            HttpResponse::json(StatusCode::OK, body)
        }
        _ => {
            let body = format!("{name} v{version}");
            HttpResponse::new(StatusCode::OK, &body)
        }
    }
}

pub fn get<F, Fut, S, R, T>(f: F, state: S) -> BoxedHandler
where
    F: Fn(HttpRequest, Query<T>, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: Into<HttpResponse>,
    S: Clone + Send + Sync + 'static,
    T: DeserializeOwned + Send + Sync + 'static,
{
    let f = Arc::new(f);

    Box::new(move |req: HttpRequest| {
        let parsed_query = Query::<T>::extract(&req);
        let state = state.clone();
        let f = Arc::clone(&f);

        Box::pin(async move {
            match parsed_query {
                Ok(query) => f(req, query, state).await.into(),
                Err(bad_request) => bad_request,
            }
        }) as BoxFuture
    })
}

pub struct Router {
    routes: HashMap<(String, String), BoxedHandler>,
    fallback_handler: Option<BoxedHandler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            fallback_handler: None,
        }
    }

    pub fn route(mut self, method: &str, path: &str, handler: BoxedHandler) -> Self {
        self.routes
            .insert((method.to_uppercase(), path.to_string()), handler);
        self
    }

    pub fn fallback<F, Fut, R>(mut self, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Into<HttpResponse>,
    {
        self.fallback_handler = Some(Box::new(move |_req: HttpRequest| {
            let fut = handler();
            Box::pin(async move { fut.await.into() }) as BoxFuture
        }));
        self
    }
}

const MAX_HEADER_SIZE: usize = 8 * 1024;

async fn read_request_headers(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];

    loop {
        let bytes_read = stream.read(&mut chunk).await?;

        if bytes_read == 0 {
            if buffer.is_empty() {
                return Ok(buffer);
            }
            bail!("Connection closed before headers were complete");
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        if buffer.len() > MAX_HEADER_SIZE {
            bail!("Request headers exceed maximum allowed size of {MAX_HEADER_SIZE} bytes");
        }

        if find_header_end(&buffer).is_some() {
            break;
        }
    }

    Ok(buffer)
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_request_line(line: &str) -> Option<(&str, &str, &str)> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let full_path = parts.next()?;
    let (path, raw_query) = match full_path.split_once('?') {
        Some((p, q)) => (p, q),
        None => (full_path, ""),
    };
    Some((method, path, raw_query))
}

pub async fn serve(listener: TcpListener, router: Router) -> anyhow::Result<()> {
    let shared_router = Arc::new(router);

    loop {
        let (mut stream, _addr) = listener.accept().await?;
        let router_clone = Arc::clone(&shared_router);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut stream, router_clone).await {
                tracing::error!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(stream: &mut TcpStream, router: Arc<Router>) -> anyhow::Result<()> {
    let raw_bytes = read_request_headers(stream).await?;
    if raw_bytes.is_empty() {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&raw_bytes);
    let mut lines = request_str.lines();
    let Some(request_line) = lines.next() else {
        return Ok(());
    };

    let Some((method, path, raw_query)) = parse_request_line(request_line) else {
        stream
            .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
            .await?;
        return Ok(());
    };

    let mut headers = HashMap::new();
    for line in lines {
        if line.is_empty() || line == "\r" {
            break;
        }
        if let Some((key, val)) = line.split_once(':') {
            let k = key.trim().to_lowercase();
            let v = val.trim().to_string();

            headers
                .entry(k)
                .and_modify(|existing: &mut String| {
                    existing.push_str(", ");
                    existing.push_str(&v);
                })
                .or_insert(v);
        }
    }

    let req = HttpRequest {
        method: method.to_uppercase(),
        path: path.to_string(),
        raw_query: raw_query.to_string(),
        headers,
    };

    let route_key = (req.method.clone(), req.path.clone());

    let response = if let Some(handler) = router.routes.get(&route_key) {
        handler(req).await
    } else {
        let path_exists_for_other_method = router
            .routes
            .keys()
            .any(|(_method, path)| path == &req.path);

        if path_exists_for_other_method {
            HttpResponse::new(StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed\n")
        } else if let Some(fallback) = router.fallback_handler.as_ref() {
            fallback(req).await
        } else {
            HttpResponse::new(StatusCode::NOT_FOUND, "404 Not Found\n")
        }
    };

    let (status_code_num, status_text) = response.status.as_parts();

    let http_response = format!(
        "HTTP/1.1 {status_code_num} {status_text}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
        response.body.len(),
        response.content_type,
        response.body
    );

    stream.write_all(http_response.as_bytes()).await?;
    Ok(())
}
