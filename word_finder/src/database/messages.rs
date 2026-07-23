use flume;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) enum DatabaseMessage {
    Search {
        word: String,
        respond_to: flume::Sender<(String, Option<HashMap<PathBuf, Vec<usize>>>)>,
    },
}
