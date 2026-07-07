use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

const TARGET_TOTAL_SIZE: usize = 100 * 1024 * 1024;
const FILE_SIZE_LIMIT: usize = 2 * 1024 * 1024;
const FILES_PER_FOLDER: usize = 5;

const TEMPLATE: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                        Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
                        nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
                        reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
                        pariatur. Excepteur sint occaecat cupidatat non proident, sunt in \
                        culpa qui officia deserunt mollit anim id est laborum.\n";

fn main() -> std::io::Result<()> {
    let base_path = Path::new("./test_data");
    let mut total_bytes_written = 0;
    let mut file_counter = 0;

    while total_bytes_written < TARGET_TOTAL_SIZE {
        let folder_index = (file_counter / FILES_PER_FOLDER) + 1;
        let current_dir = base_path.join(format!("folder_{folder_index}"));

        if file_counter % FILES_PER_FOLDER == 0 {
            fs::create_dir_all(&current_dir)?;
        }

        file_counter += 1;
        let file_path = current_dir.join(format!("text_{file_counter}.txt"));

        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        let mut current_file_bytes = 0;

        while current_file_bytes < FILE_SIZE_LIMIT && total_bytes_written < TARGET_TOTAL_SIZE {
            writer.write_all(TEMPLATE)?;
            current_file_bytes += TEMPLATE.len();
            total_bytes_written += TEMPLATE.len();
        }

        writer.flush()?;
    }

    Ok(())
}
