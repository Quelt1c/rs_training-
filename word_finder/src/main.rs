mod case_checker;
use clap::Parser;
mod channel;
mod io_utils;
mod network_handler;
mod server;
mod spawn_workers;
mod text_tools;
use crate::case_checker::Checker;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{Level, info};

enum DatabaseMessage {
    Search {
        word: String,
        respond_to: channel::Sender<(String, Option<HashMap<PathBuf, Vec<usize>>>)>,
    },
    InsertReport {
        file_path: PathBuf,
        words_map: HashMap<String, Vec<usize>>,
    },
}

#[derive(Clone)]
struct Database {
    sender: channel::Sender<DatabaseMessage>,
}

impl Database {
    fn search(
        &self,
        word: String,
        respond_to: channel::Sender<(String, Option<HashMap<PathBuf, Vec<usize>>>)>,
    ) {
        let _ = self
            .sender
            .send(DatabaseMessage::Search { word, respond_to });
    }

    fn insert_report(&self, file_path: PathBuf, words_map: HashMap<String, Vec<usize>>) {
        let _ = self.sender.send(DatabaseMessage::InsertReport {
            file_path,
            words_map,
        });
    }
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let args: Checker = Checker::parse();

    let (db_tx, db_rx) = channel::unbounded();
    let db = Database { sender: db_tx };

    std::thread::spawn(move || {
        let mut storage = HashMap::new();

        while let Ok(msg) = db_rx.recv() {
            match msg {
                DatabaseMessage::Search { word, respond_to } => {
                    let res = storage.get(&word).cloned();
                    let _ = respond_to.send((word, res));
                }
                DatabaseMessage::InsertReport {
                    file_path,
                    words_map,
                } => {
                    for (word, indices) in words_map {
                        storage
                            .entry(word)
                            .or_default()
                            .insert(file_path.clone(), indices);
                    }
                }
            }
        }
    });

    info!("Working {} threads", args.threads);

    let (input_tx, input_rx) = channel::unbounded();
    let (output_tx, output_rx) = channel::unbounded();

    spawn_workers::spawn_worker_threads(
        input_rx,
        output_tx,
        args.case_sensitive,
        args.threads.get(),
    );

    io_utils::produce_file_tasks(&args.file_path, input_tx)?;

    while let Ok(report) = output_rx.recv() {
        db.insert_report(report.file_path, report.words_map);
    }

    info!("Data indexing completed");

    server::run_server(db, args.case_sensitive, "127.0.0.1:27015")?;

    Ok(())
}
