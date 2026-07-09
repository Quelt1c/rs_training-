use clap::Parser;
use std::{num::NonZeroUsize, path::PathBuf};

#[derive(Parser, Debug)]

pub struct Checker {
    #[arg(short, long)]
    pub case_sensitive: bool,
    pub file_path: PathBuf,
    #[arg(short, long, default_value_t = NonZeroUsize::new(1).unwrap())]
    pub threads: NonZeroUsize,
}
