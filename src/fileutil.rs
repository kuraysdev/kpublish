use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_first_line(file_path: &str) -> Option<String> {
    if let Ok(file) = File::open(file_path) {
        let reader = BufReader::new(file);

        if let Some(line) = reader.lines().next() {
            return Some(line.unwrap_or("none".to_owned()));
        }
    }

    None
}