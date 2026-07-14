use crate::channel;
use crate::text_tools::parser::split_by_word_own;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
pub struct FileReport {
    pub file_path: PathBuf,
    pub words_map: HashMap<String, Vec<usize>>,
}
fn parse_and_normalize_file(
    current: &Path,
    case_sensitive: bool,
) -> Option<HashMap<String, Vec<usize>>> {
    if current.is_file()
        && current.extension().map_or(false, |ext| ext == "txt")
        && let Ok(text) = std::fs::read_to_string(current)
    {
        let mut raw_map = HashMap::new();
        split_by_word_own(&mut raw_map, &text);

        let mut processed_map: HashMap<String, Vec<usize>> = HashMap::with_capacity(raw_map.len());

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
    None
}

pub fn produce_file_tasks(path: &Path, input_tx: channel::Sender<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        let _ = input_tx.send(path.to_path_buf());
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let current = entry.path();

        if current.is_dir() {
            produce_file_tasks(&current, input_tx.clone())?;
        } else {
            let _ = input_tx.send(current);
        }
    }
    Ok(())
}

pub fn file_worker_channels(
    input_rx: channel::Receiver<PathBuf>,
    output_tx: channel::Sender<FileReport>,
    case_sensitive: bool,
) {
    while let Ok(current) = input_rx.recv() {
        if let Some(processed_map) = parse_and_normalize_file(&current, case_sensitive) {
            let report = FileReport {
                file_path: current,
                words_map: processed_map,
            };

            if output_tx.send(report).is_err() {
                break;
            }
        }
    }
}
