use crate::database::messages::DatabaseMessage;
use flume;
use std::collections::HashMap;
use std::path::PathBuf;

mod io_utils;
mod messages;
mod spawn_workers;

use io_utils::{FileReport, produce_file_tasks};
use spawn_workers::spawn_worker_threads;

#[derive(Clone)]
pub struct Database {
    sender: flume::Sender<DatabaseMessage>,
    case_sensitive: bool,
}

impl Database {
    pub fn new(file_path: PathBuf, threads: usize, case_sensitive: bool) -> Self {
        let (tx, rx) = flume::unbounded();

        tokio::spawn(async move {
            let mut storage: HashMap<String, HashMap<PathBuf, Vec<usize>>> = HashMap::new();

            let (input_tx, input_rx) = flume::unbounded();
            let (output_tx, output_rx) = flume::unbounded();

            spawn_worker_threads(input_rx, output_tx, case_sensitive, threads);

            if let Err(e) = produce_file_tasks(file_path, input_tx).await {
                tracing::error!("File scanning error: {}", e);
            }

            while let Ok(report) = output_rx.recv_async().await {
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

            while let Ok(msg) = rx.recv_async().await {
                match msg {
                    DatabaseMessage::Search { word, respond_to } => {
                        let res = storage.get(&word).cloned();
                        let _ = respond_to.send_async((word, res)).await;
                    }
                }
            }
        });

        Self {
            sender: tx,
            case_sensitive,
        }
    }

    pub async fn get(&self, word: String) -> Option<HashMap<PathBuf, Vec<usize>>> {
        let search_word = if self.case_sensitive {
            word
        } else {
            word.to_lowercase()
        };

        let (response_tx, response_rx) = flume::bounded(1);

        let send_result = self
            .sender
            .send_async(DatabaseMessage::Search {
                word: search_word,
                respond_to: response_tx,
            })
            .await;

        if send_result.is_err() {
            return None;
        }

        match response_rx.recv_async().await {
            Ok((_returned_word, result)) => result,
            Err(_) => None,
        }
    }
}
