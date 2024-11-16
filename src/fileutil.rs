use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use pulldown_cmark::{Parser, Options, html};
use std::io::Read;
use crate::render;
use regex::Regex;
use chrono::{DateTime, Local};

// Struct for file or directory representation
#[derive(Serialize)]
pub struct FileNode {
    name: String,
    path: String,
    is_directory: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostMetadata {
    pub title: Option<String>,
    pub template: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize)]
pub struct PostInfo {
    pub path: String,
    pub metadata: serde_json::Value, // Change to a serializable type
    pub preview: String,
    pub date: String
}

#[derive(Serialize)]
pub struct DirectoryIndex {
    pub posts: Vec<PostInfo>,
    pub subdirs: Vec<String>,
}

fn get_file_date(path: &Path) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            let datetime: DateTime<Local> = modified.into();
            return datetime.format("%Y-%m-%d").to_string();
        }
    }
    "1970-01-01".to_string() // fallback date if we can't get modification time
}

fn extract_preview(content: &str) -> String {
    // Parse markdown
    let parser = Parser::new_ext(content, Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Strip HTML tags and get first few sentences
    let re = Regex::new(r"<[^>]*>").unwrap();

    let text = re.replace_all(&html_output, "").to_string();
    let preview = text.split('.')
        .take(2)  // Take first two sentences
        .collect::<Vec<&str>>()
        .join(". ");
    
    if preview.is_empty() {
        content.chars().take(200).collect()
    } else {
        preview + "..."
    }
}

pub fn get_directory_index(dir_path: &Path) -> DirectoryIndex {
    let mut posts = Vec::new();
    let mut subdirs = Vec::new();

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(dir_str) = dir_name.to_str() {
                            if !dir_str.starts_with('.') {  // Skip hidden directories
                                subdirs.push(dir_str.to_string());
                            }
                        }
                    }
                } else if let Some(extension) = path.extension() {
                    if extension == "md" {
                        if let Ok(mut file) = fs::File::open(&path) {
                            let mut contents = String::new();
                            if file.read_to_string(&mut contents).is_ok() {

                                let (headers, content) = render::get_headers(&contents);
                                // Parse frontmatter and content
                                if let Ok(metadata) = serde_yaml::from_str::<render::Headers>(&headers) {
                                    let preview = extract_preview(&content);
                                    
                                    // Get relative path
                                    let rel_path = path.strip_prefix("public")
                                        .unwrap_or(&path)
                                        .to_string_lossy()
                                        .into_owned();

                                    // Get date from metadata or file modification time
                                    let date = get_file_date(&path);
                                    
                                    // Convert Headers to serde_json::Value
                                    let metadata_json = serde_json::to_value(metadata).unwrap_or_default();
                                    
                                    posts.push(PostInfo {
                                        path: rel_path,
                                        metadata: metadata_json,
                                        preview,
                                        date
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort posts by date if available, most recent first
    // posts.sort_by(|a, b| {
    //     b.metadata..date.as_deref()
    //         .unwrap_or("")
    //         .cmp(&a.metadata.date.as_deref().unwrap_or(""))
    // });

    DirectoryIndex { posts, subdirs }
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
                let path = path_buf.to_string_lossy().to_string().replace("\\","/");

                if !is_directory {
                    result.push(FileNode {
                        name,
                        path,
                        is_directory,
                    });
                }
                
                if is_directory {
                    let subfolder_entries = traverse_folder(&path_buf);
                    result.extend(subfolder_entries);
                }
            }
        }
    }

    result
}