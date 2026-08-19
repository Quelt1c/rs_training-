// use crate::database::Database;
// use std::collections::HashMap;
// use std::path::PathBuf;
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// use tokio::net::TcpStream;
// use tracing::info;

// pub async fn handle_network_client(
//     mut stream: TcpStream,
//     db: Database,
//     client_id: usize,
// ) -> std::io::Result<()> {
//     let client_addr = stream
//         .peer_addr()
//         .map(|addr| addr.to_string())
//         .unwrap_or_else(|_| "Unknown".to_string());

//     info!("Client #{client_id} connected from {client_addr}");

//     let (reader, mut writer) = stream.split();
//     let mut buf_reader = BufReader::new(reader);
//     let mut request_line = String::new();

//     buf_reader.read_line(&mut request_line).await?;
//     if request_line.is_empty() {
//         return Ok(());
//     }

//     let mut line = String::new();
//     loop {
//         line.clear();
//         buf_reader.read_line(&mut line).await?;
//         if line.trim().is_empty() {
//             break;
//         }
//     }

//     let parts: Vec<&str> = request_line.split_whitespace().collect();
//     if parts.len() < 2 {
//         return Ok(());
//     }
//     let path = parts[1];

//     let (status_line, content) = if let Some(raw_word) = path.strip_prefix("/search?word=") {
//         let clean_word = raw_word.trim();

//         if clean_word.is_empty() {
//             (
//                 "HTTP/1.1 200 OK",
//                 "Usage: enter a word in the query parameter (e.g., http://127.0.0.1:27015/search?word=Lorem)\n"
//                     .to_string(),
//             )
//         } else {
//             let results = db.get(clean_word.to_string());

//             (
//                 "HTTP/1.1 200 OK",
//                 format_results(clean_word.to_string(), results.await),
//             )
//         }
//     } else {
//         (
//             "HTTP/1.1 404 NOT FOUND",
//             "404 Not Found. Please use format: /search?word=YOUR_WORD\n".to_string(),
//         )
//     };

//     let response = format!(
//         "{}\r\n\
//         Content-Type: text/plain; charset=utf-8\r\n\
//         Content-Length: {}\r\n\
//         Connection: close\r\n\
//         \r\n\
//         {}",
//         status_line,
//         content.as_bytes().len(),
//         content
//     );

//     writer.write_all(response.as_bytes()).await?;
//     writer.flush().await?;

//     info!("Client №{client_id} finished from {client_addr}.");
//     Ok(())
// }

// fn format_results(word: String, results: Option<HashMap<PathBuf, Vec<usize>>>) -> String {
//     let mut output = String::new();
//     output.push_str(&format!("Search results for: {word}\n"));
//     output.push_str("----------------------------------------\n");

//     match results {
//         Some(files) if !files.is_empty() => {
//             for (path, indices) in files {
//                 output.push_str(&format!(
//                     "File: {}\nOccurrences: {}\nIndices: {:?}\n\n",
//                     path.to_string_lossy(),
//                     indices.len(),
//                     indices
//                 ));
//             }
//         }
//         _ => {
//             output.push_str("No results found or background indexing is still in progress.\n");
//         }
//     }

//     output
// }

use super::own_axum::{ContentType, HttpMethod, HttpRequest, HttpResponse, Query, StatusCode};
use crate::database::Database;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    word: String,
}

pub async fn search_handler(
    req: HttpRequest,
    Query(params): Query<SearchParams>,
    db: Database,
) -> HttpResponse {
    let accepts_json = req
        .header("accept")
        .is_some_and(|v| v.to_lowercase().contains("application/json"));
    let word_source = if req.method() == HttpMethod::POST
        && req.content_type() == Some(ContentType::ApplicationJson)
    {
        match serde_json::from_str::<SearchParams>(req.body()) {
            Ok(body_params) => body_params.word,
            Err(_) => {
                let msg = "Invalid JSON body. Expected: {\"word\": \"...\"}";
                return if accepts_json {
                    HttpResponse::json(StatusCode::BAD_REQUEST, format!(r#"{{"error": "{msg}"}}"#))
                } else {
                    HttpResponse::new(StatusCode::BAD_REQUEST, msg)
                };
            }
        }
    } else {
        params.word
    };

    let clean_word = word_source.trim();

    if clean_word.is_empty() {
        let msg = "Usage: enter a word in the query parameter (e.g., http://127.0.0.1:27015/search?word=Lorem) or send POST with JSON body {\"word\": \"Lorem\"}";
        return if accepts_json {
            HttpResponse::json(StatusCode::BAD_REQUEST, format!(r#"{{"error": "{msg}"}}"#))
        } else {
            HttpResponse::new(StatusCode::BAD_REQUEST, msg)
        };
    }

    let results = db.get(clean_word.to_string()).await;

    match results {
        Some(files) if !files.is_empty() => {
            if accepts_json {
                let json_body = match serde_json::to_string(&files) {
                    Ok(json) => json,
                    Err(_) => "{}".to_string(),
                };
                HttpResponse::json(StatusCode::OK, json_body)
            } else {
                let text_body = format_results(clean_word, files);
                HttpResponse::new(StatusCode::OK, &text_body)
            }
        }
        _ => {
            if accepts_json {
                HttpResponse::json(
                    StatusCode::NOT_FOUND,
                    r#"{"error": "No results found"}"#.to_string(),
                )
            } else {
                HttpResponse::new(StatusCode::NOT_FOUND, "No results found\n")
            }
        }
    }
}

fn format_results(word: &str, files: HashMap<PathBuf, Vec<usize>>) -> String {
    let mut output = String::new();
    output.push_str(&format!("Search results for: {word}\n"));

    for (path, indices) in files {
        output.push_str(&format!(
            "File: {}\nOccurrences: {}\nIndices: {:?}\n\n",
            path.to_string_lossy(),
            indices.len(),
            indices
        ));
    }

    output
}
