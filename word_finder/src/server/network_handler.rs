use crate::database::Database;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use tracing::info;

#[derive(Serialize)]
struct NetworkResponse {
    word: String,
    found: bool,
    results: HashMap<String, Vec<usize>>,
}

pub fn handle_network_client(
    stream: TcpStream,
    db: Database,
    case_sensitive: bool,
    client_id: usize,
) -> std::io::Result<()> {
    info!("Client #{} connected!", client_id);

    let mut reader = BufReader::new(stream);

    {
        let socket = reader.get_mut();
        writeln!(socket, "{}", client_id)?;
        socket.flush()?;
    }

    loop {
        let mut input_line = String::new();
        let bytes_read = reader.read_line(&mut input_line)?;
        if bytes_read == 0 {
            break;
        }

        let trimmed_word = input_line.trim();
        if trimmed_word == "q" || trimmed_word.is_empty() {
            break;
        }

        let search_word = if case_sensitive {
            trimmed_word.to_string()
        } else {
            trimmed_word.to_lowercase()
        };

        let results = db.get(search_word.clone());

        let response = match results {
            Some(files_with_word) => {
                let mut string_results = HashMap::new();
                for (path, indices) in files_with_word {
                    string_results.insert(path.to_string_lossy().into_owned(), indices);
                }
                NetworkResponse {
                    word: search_word,
                    found: true,
                    results: string_results,
                }
            }
            None => NetworkResponse {
                word: search_word,
                found: false,
                results: HashMap::new(),
            },
        };

        if let Ok(json_string) = serde_json::to_string(&response) {
            let mut payload = json_string;
            payload.push('\n');

            let socket = reader.get_mut();
            if socket.write_all(payload.as_bytes()).is_err() || socket.flush().is_err() {
                break;
            }
        }
    }

    info!("Client #{} disconnected.", client_id);
    Ok(())
}
