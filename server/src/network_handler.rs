use serde::Serialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize)]
struct NetworkResponse {
    word: String,
    found: bool,
    results: HashMap<String, Vec<usize>>,
}
pub fn handle_network_client(
    stream: TcpStream,
    map: Arc<HashMap<String, HashMap<PathBuf, Vec<usize>>>>,
    case_sensitive: bool,
    client_id: usize,
) -> std::io::Result<()> {
    println!("Client #{} connected!", client_id);

    let mut reader = BufReader::new(stream);

    let socket = reader.get_mut();
    writeln!(socket, "{}", client_id)?;
    socket.flush()?;

    loop {
        let mut input_line = String::new();

        let bytes_read = reader.read_line(&mut input_line)?;
        if bytes_read == 0 {
            println!("Client #{} leaves.", client_id);
            break;
        }

        let trimmed_word = input_line.trim();

        if trimmed_word == "q" || trimmed_word.is_empty() {
            println!("Client #{} closed the session.", client_id);
            break;
        }

        let search_word = if case_sensitive {
            trimmed_word.to_string()
        } else {
            trimmed_word.to_lowercase()
        };

        let response = if let Some(files_with_word) = map.get(&search_word) {
            let mut string_results = HashMap::new();
            for (path, indices) in files_with_word {
                string_results.insert(path.to_string_lossy().into_owned(), indices.clone());
            }

            NetworkResponse {
                word: trimmed_word.to_string(),
                found: true,
                results: string_results,
            }
        } else {
            NetworkResponse {
                word: trimmed_word.to_string(),
                found: false,
                results: HashMap::new(),
            }
        };

        let mut json_string = serde_json::to_string(&response).unwrap();
        json_string.push('\n');

        println!(
            "[Server -> Client #{}]: Sending the answear: {}",
            client_id,
            json_string.trim()
        );

        let socket = reader.get_mut();
        socket.write_all(json_string.as_bytes())?;
        socket.flush()?;
    }

    Ok(())
}
