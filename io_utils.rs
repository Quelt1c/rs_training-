use std::fs;
use std::io;

pub fn read_file(file_path: &str) -> io::Result<String> {
    let text_from_file = fs::read_to_string(file_path)?;
    Ok(text_from_file)
}