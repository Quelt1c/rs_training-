use super::http_method::HttpMethod;
use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use super::router::Router;
use super::status_code::StatusCode;
use anyhow::bail;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

async fn read_request_headers(stream: &mut TcpStream) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];

    loop {
        let bytes_read = stream.read(&mut chunk).await?;

        if bytes_read == 0 {
            if buffer.is_empty() {
                return Ok((Vec::new(), Vec::new()));
            }
            bail!("Connection closed before headers were complete");
        }

        buffer.extend_from_slice(&chunk[..bytes_read]);

        if buffer.len() > 8 * 1024 {
            bail!("Request headers exceed 8KB limit");
        }

        if let Some(pos) = find_header_end(&buffer) {
            let headers_bytes = buffer[..pos].to_vec();
            let extra_body_bytes = buffer[pos..].to_vec();
            return Ok((headers_bytes, extra_body_bytes));
        }
    }
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

async fn dispatch(router: &Router, req: HttpRequest) -> HttpResponse {
    let route_key = (req.method(), req.path().to_string());

    if let Some(handler) = router.routes.get(&route_key) {
        return handler(req).await;
    }

    let path_exists_for_other_method = router
        .routes
        .keys()
        .any(|(_method, route_path)| route_path == req.path());

    if path_exists_for_other_method {
        return HttpResponse::new(StatusCode::METHOD_NOT_ALLOWED, "405 Method Not Allowed\n");
    }

    if let Some(fallback) = router.fallback_handler.as_ref() {
        return fallback(req).await;
    }

    HttpResponse::new(StatusCode::NOT_FOUND, "404 Not Found\n")
}

async fn handle_connection(stream: &mut TcpStream, router: Arc<Router>) -> anyhow::Result<()> {
    let (raw_headers_bytes, mut extra_body_bytes) = read_request_headers(stream).await?;
    if raw_headers_bytes.is_empty() {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&raw_headers_bytes);
    let mut lines = request_str.lines();
    let Some(request_line) = lines.next() else {
        return Ok(());
    };

    let Some((method_str, path, raw_query)) = parse_request_line(request_line) else {
        stream
            .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
            .await?;
        return Ok(());
    };

    let Some(method) = HttpMethod::from_str(method_str) else {
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

    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    while extra_body_bytes.len() < content_length {
        let mut chunk = [0u8; 1024];
        let bytes_read = stream.read(&mut chunk).await?;
        if bytes_read == 0 {
            break;
        }
        extra_body_bytes.extend_from_slice(&chunk[..bytes_read]);
    }

    let body =
        String::from_utf8_lossy(&extra_body_bytes[..content_length.min(extra_body_bytes.len())])
            .to_string();
    let parsed_query_map: HashMap<String, String> =
        serde_urlencoded::from_str(raw_query).unwrap_or_default();

    let req = HttpRequest::new(
        method,
        path.to_string(),
        raw_query.to_string(),
        parsed_query_map,
        headers,
        body,
    );

    let response = dispatch(&router, req).await;

    let (status_code_num, status_text) = response.status.as_parts();
    let mut response_bytes = format!("HTTP/1.1 {status_code_num} {status_text}\r\n").into_bytes();

    let mut response_headers = response.headers.clone();
    response_headers
        .entry("content-length".to_string())
        .or_insert_with(|| response.body.len().to_string());

    for (key, val) in &response_headers {
        response_bytes.extend_from_slice(format!("{key}: {val}\r\n").as_bytes());
    }

    response_bytes.extend_from_slice(b"\r\n");
    response_bytes.extend_from_slice(&response.body);

    stream.write_all(&response_bytes).await?;
    Ok(())
}
