use serde::Serialize;
use std::fs;
use std::path::Path;
// Struct for file or directory representation
#[derive(Serialize)]
pub struct FileNode {
    name: String,
    path: String,
    is_directory: bool,
}

// Function to recursively traverse folder structure and collect file info
pub fn traverse_folder(path: &Path) -> Vec<FileNode> {
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_type = entry.file_type().ok();
                let is_directory = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
                let name = entry.file_name().into_string().unwrap_or_else(|_| String::from("INVALID_NAME"));
                let path_buf = entry.path();
                let path = path_buf.to_string_lossy().to_string();

                result.push(FileNode {
                    name,
                    path,
                    is_directory,
                });

                if is_directory {
                    let subfolder_entries = traverse_folder(&path_buf);
                    result.extend(subfolder_entries);
                }
            }
        }
    }

    result
}