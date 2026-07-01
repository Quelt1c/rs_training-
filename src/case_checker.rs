use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]

pub struct Checker {
    #[arg(short, long)]
    pub case_sensitive: bool,
    pub file_path: PathBuf,
    pub text: String,
}
