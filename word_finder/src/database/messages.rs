use crate::channel;
use std::collections::HashMap;
use std::path::PathBuf;
pub(crate) enum DatabaseMessage {
    Search {
        word: String,
        respond_to: channel::Sender<(String, Option<HashMap<PathBuf, Vec<usize>>>)>,
    },
    InsertReport {
        file_path: PathBuf,
        words_map: HashMap<String, Vec<usize>>,
    },
}
