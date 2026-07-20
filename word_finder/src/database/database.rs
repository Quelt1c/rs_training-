use super::messages::DatabaseMessage;
use crate::channel;
use crate::io_utils::FileReport;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Database {
    sender: channel::Sender<DatabaseMessage>,
}

impl Database {
    pub fn new(file_path: PathBuf, threads: usize, case_sensitive: bool) -> Self {
        let (tx, rx) = channel::unbounded();

        std::thread::spawn(move || {
            let mut storage: HashMap<String, HashMap<PathBuf, Vec<usize>>> = HashMap::new();

            let (input_tx, input_rx) = channel::unbounded();
            let (output_tx, output_rx) = channel::unbounded();

            crate::pipeline::spawn_workers::spawn_worker_threads(
                input_rx,
                output_tx,
                case_sensitive,
                threads,
            );

            if let Err(e) = crate::io_utils::produce_file_tasks(&file_path, input_tx) {
                tracing::error!("File scanning error: {}", e);
            }

            while let Ok(report) = output_rx.recv() {
                let FileReport {
                    file_path,
                    words_map,
                } = report;

                for (word, indices) in words_map {
                    storage
                        .entry(word)
                        .or_default()
                        .insert(file_path.clone(), indices);
                }
            }

            tracing::info!("Background indexing of the database completed successfully!");

            while let Ok(msg) = rx.recv() {
                match msg {
                    DatabaseMessage::Search { word, respond_to } => {
                        let res = storage.get(&word).cloned();
                        let _ = respond_to.send((word, res));
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub fn get(&self, word: String) -> Option<HashMap<PathBuf, Vec<usize>>> {
        let (response_tx, response_rx) = channel::unbounded();

        let send_result = self.sender.send(DatabaseMessage::Search {
            word,
            respond_to: response_tx,
        });

        if send_result.is_err() {
            return None;
        }
        match response_rx.recv() {
            Ok((_returned_word, result)) => result,
            Err(_) => None,
        }
    }
}
