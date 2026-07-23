use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use tracing::{Level, error, info};

const SERVER_ADDR: &str = "127.0.0.1:27015";

#[derive(Debug, Deserialize)]
struct NetworkResponse {
    word: String,
    found: bool,
    results: HashMap<String, Vec<usize>>,
}

fn main() -> io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("Connecting to the server{}...", SERVER_ADDR);
    let stream = TcpStream::connect(SERVER_ADDR)?;
    info!("Connected succesfully!");

    let mut server_reader = BufReader::new(&stream);
    let mut server_writer = &stream;

    let mut stdin_reader = BufReader::new(io::stdin());

    let mut id_line = String::new();
    if server_reader.read_line(&mut id_line)? == 0 {
        error!("Error: The server closed the connection before the ID was sent.");
        return Ok(());
    }
    let client_id = id_line.trim().to_string();

    info!("You're №{}", client_id);
    info!("Enter the word or q to exit");

    loop {
        info!("\n[Client #{}] Searching word :", client_id);
        io::stdout().flush()?;

        let mut input = String::new();
        stdin_reader.read_line(&mut input)?;

        let trimmed = input.trim();
        if trimmed == "q" || trimmed.is_empty() {
            info!("Client №{} shut down.", client_id);
            break;
        }

        server_writer.write_all(input.as_bytes())?;
        server_writer.flush()?;

        let mut response_line = String::new();
        if server_reader.read_line(&mut response_line)? == 0 {
            error!("Connection interrupted: the server closed the communication flume.");
            break;
        }

        match serde_json::from_str::<NetworkResponse>(&response_line) {
            Ok(res) if res.found => {
                println!("Result for the word[{}]:", res.word);
                for (file_path, indices) in res.results {
                    println!("     File: {}", file_path);
                    println!("     Indices: {:?}", indices);
                }
            }
            Ok(res) => {
                info!(" Word [{}] not found.", res.word);
            }
            Err(e) => {
                error!("Error of deserialization json: {}", e);
                error!("Invalid data received: {}", response_line.trim());
            }
        }
    }

    Ok(())
}
