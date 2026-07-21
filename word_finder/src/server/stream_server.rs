use crate::database::Database;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use tracing::info;

pub fn handle_network_client(
    stream: TcpStream,
    db: Database,
    case_sensitive: bool,
    client_id: usize,
) -> std::io::Result<()> {
    info!("Client #{} connected!", client_id);

    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();

    reader.read_line(&mut request_line)?;
    if request_line.is_empty() {
        return Ok(());
    }

    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }
    let path = parts[1];

    let (status_line, content) = if path == "/" {
        (
            "HTTP/1.1 200 OK",
            "Usage: enter a word directly in the URL (e.g., http://127.0.0.1:27015/Lorem)\n"
                .to_string(),
        )
    } else {
        let raw_word = path
            .trim_start_matches("/search?word=")
            .trim_start_matches('/');

        let search_word = if case_sensitive {
            raw_word.to_string()
        } else {
            raw_word.to_lowercase()
        };

        let results = db.get(search_word.clone());

        ("HTTP/1.1 200 OK", format_results(search_word, results))
    };

    let socket = reader.get_mut();
    let response = format!(
        "{}\r\n\
        Content-Type: text/plain; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        status_line,
        content.len(),
        content
    );

    socket.write_all(response.as_bytes())?;
    socket.flush()?;

    info!("Client №{} finished.", client_id);
    Ok(())
}

fn format_results(word: String, results: Option<HashMap<PathBuf, Vec<usize>>>) -> String {
    let mut output = String::new();
    output.push_str(&format!("Search results for: {}\n", word));
    output.push_str("----------------------------------------\n");

    match results {
        Some(files) if !files.is_empty() => {
            for (path, indices) in files {
                output.push_str(&format!(
                    "File: {}\nOccurrences: {}\nIndices: {:?}\n\n",
                    path.to_string_lossy(),
                    indices.len(),
                    indices
                ));
            }
        }
        _ => {
            output.push_str("No results found or background indexing is still in progress.\n");
        }
    }

    output
}
