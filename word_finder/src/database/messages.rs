use crate::channel;
use crate::database::{IndexedFile, WordMatch};
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) enum DatabaseMessage {
    SearchWord {
        word: String,
        respond_to: channel::Sender<Vec<WordMatch>>,
    },
    ListWords {
        respond_to: channel::Sender<Vec<String>>,
    },
    ListFiles {
        respond_to: channel::Sender<Vec<IndexedFile>>,
    },
    FileWords {
        file_id: u64,
        respond_to: channel::Sender<Option<HashMap<String, Vec<usize>>>>,
    },
    AddPath {
        path: PathBuf,
        respond_to: channel::Sender<Result<Vec<IndexedFile>, String>>,
    },
}
