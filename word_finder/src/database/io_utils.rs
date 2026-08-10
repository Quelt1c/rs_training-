use crate::text_tools::parser::split_by_word_own;
use flume;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FileReport {
    pub file_path: PathBuf,
    pub words_map: HashMap<String, Vec<usize>>,
}

pub async fn produce_file_tasks(
    path: PathBuf,
    input_tx: flume::Sender<PathBuf>,
) -> std::io::Result<()> {
    let mut dirs_to_visit = vec![path];

    while let Some(current) = dirs_to_visit.pop() {
        let metadata = tokio::fs::metadata(&current).await?;

        if metadata.is_file() {
            let _ = input_tx.send_async(current).await;
            continue;
        }

        let mut entries = tokio::fs::read_dir(current).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_dir() {
                    dirs_to_visit.push(entry_path);
                } else {
                    let _ = input_tx.send_async(entry_path).await;
                }
            }
        }
    }
    Ok(())
}

pub async fn file_worker_flumes(
    input_rx: flume::Receiver<PathBuf>,
    output_tx: flume::Sender<FileReport>,
    case_sensitive: bool,
) {
    while let Ok(current) = input_rx.recv_async().await {
        if let Some(processed_map) = parse_and_normalize_file(&current, case_sensitive).await {
            let report = FileReport {
                file_path: current.to_path_buf(),
                words_map: processed_map,
            };

            if output_tx.send_async(report).await.is_err() {
                break;
            }
        }
    }
}

async fn parse_and_normalize_file(
    current: &Path,
    case_sensitive: bool,
) -> Option<HashMap<String, Vec<usize>>> {
    if current.extension().map_or(false, |ext| ext == "txt") {
        if let Ok(text) = tokio::fs::read_to_string(current).await {
            let mut raw_map = HashMap::new();

            split_by_word_own(&mut raw_map, &text);

            let mut processed_map: HashMap<String, Vec<usize>> =
                HashMap::with_capacity(raw_map.len());

            for (word, indices) in raw_map {
                let processed_word = if case_sensitive {
                    word
                } else {
                    word.to_lowercase()
                };
                processed_map.insert(processed_word, indices);
            }
            return Some(processed_map);
        }
    }
    None
}
