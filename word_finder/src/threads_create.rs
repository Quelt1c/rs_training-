use crate::io_utils::walk_dir;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
type SharedMap = Arc<Mutex<HashMap<String, HashMap<PathBuf, Vec<usize>>>>>;

pub fn spawn_search_threads(
    root_path: &Path,
    shared_map: SharedMap,
    case_sensitive: bool,
) -> std::io::Result<Vec<JoinHandle<()>>> {
    let mut entries = Vec::new();

    if root_path.is_dir() {
        for entry in std::fs::read_dir(root_path)? {
            entries.push(entry?.path());
        }
    } else {
        entries.push(root_path.to_path_buf());
    }

    if entries.is_empty() {
        return Ok(Vec::new());
    }

    let chunk_size = (entries.len() + 3) / 4;
    let mut handles = Vec::new();

    for chunk in entries.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let map_clone = Arc::clone(&shared_map);

        let handle = thread::spawn(move || {
            for path in chunk {
                let _ = walk_dir(&path, Arc::clone(&map_clone), case_sensitive);
            }
        });
        handles.push(handle);
    }
    Ok(handles)
}
