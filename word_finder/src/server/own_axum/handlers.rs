use super::content_type::ContentType;
use super::http_method::HttpMethod;
use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use super::query::Query;
use super::status_code::StatusCode;
use crate::database::Database;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct InfoParams {
    pub format: Option<String>,
}

pub async fn info_handler(
    req: HttpRequest,
    query: Query<InfoParams>,
    _db: Database,
) -> HttpResponse {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");

    let accepts_json = req
        .header("accept")
        .map(|a| a.to_lowercase().contains("application/json"))
        .unwrap_or(false);

    if query.0.format.as_deref() == Some("json") || accepts_json {
        let body = format!(r#"{{"name": "{name}", "version": "{version}"}}"#);
        HttpResponse::json(StatusCode::OK, body)
    } else {
        let body = format!("{name} v{version}");
        HttpResponse::new(StatusCode::OK, &body)
    }
}

#[derive(Deserialize)]
pub struct DownloadParams {
    pub file: Option<String>,
}

fn respond_error(status: StatusCode, msg: &str, as_json: bool) -> HttpResponse {
    if as_json {
        HttpResponse::json(status, format!(r#"{{"error": "{msg}"}}"#))
    } else {
        HttpResponse::new(status, msg)
    }
}

pub async fn download_handler(
    req: HttpRequest,
    query: Query<DownloadParams>,
    _db: Database,
) -> HttpResponse {
    let accepts_json = req
        .header("accept")
        .map(|a| a.to_lowercase().contains("application/json"))
        .unwrap_or(false);

    let file_param = if req.method() == HttpMethod::POST
        && req.content_type() == Some(ContentType::ApplicationJson)
    {
        match serde_json::from_str::<DownloadParams>(req.body()) {
            Ok(body_params) => body_params.file,
            Err(_) => {
                let msg = "Invalid JSON body. Expected: {\"file\": \"path/to/file\"}";
                return respond_error(StatusCode::BAD_REQUEST, msg, accepts_json);
            }
        }
    } else {
        query
            .0
            .file
            .or_else(|| req.query_param("file").map(String::from))
    };

    let file_param = match file_param {
        Some(f) if !f.trim().is_empty() => f,
        _ => {
            let msg = "Missing 'file' parameter. Use GET /download?file=path or POST /download with JSON body {\"file\": \"path\"}";
            return respond_error(StatusCode::BAD_REQUEST, msg, accepts_json);
        }
    };

    let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let requested_path = PathBuf::from(&file_param);

    let file_path = if requested_path.is_absolute() {
        requested_path
    } else {
        base_dir.join(&requested_path)
    };

    if !file_path.exists() || !file_path.is_file() {
        return respond_error(StatusCode::NOT_FOUND, "File not found\n", accepts_json);
    }

    match fs::read(&file_path) {
        Ok(bytes) => {
            let filename = file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("downloaded_file");
            HttpResponse::file(filename, bytes)
        }
        Err(e) => respond_error(
            StatusCode::BAD_REQUEST,
            &format!("Failed to read file: {e}\n"),
            accepts_json,
        ),
    }
}
