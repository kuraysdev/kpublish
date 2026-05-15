use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::render;

#[derive(Deserialize, Default)]
struct Frontmatter {
    title: Option<String>,
    date: Option<String>,
    description: Option<String>,
    password: Option<String>,
}

struct FeedItem {
    title: String,
    link: String,
    description: String,
    pub_date: String,
    timestamp: i64,
}

pub fn build_rss_feed<P: AsRef<Path>>(
    public_dir: P,
    base_url: &str,
    section_path: &str,
) -> Result<String, String> {
    let public_dir = public_dir.as_ref();
    let section_path = section_path.trim_matches('/');
    let section_dir = if section_path.is_empty() {
        public_dir.to_path_buf()
    } else {
        public_dir.join(section_path)
    };

    if !section_dir.exists() || !section_dir.is_dir() {
        return Err(format!("NOT_FOUND: section not found {}", section_dir.display()));
    }

    let index_path = section_dir.join("index.md");
    if !index_path.exists() || !index_path.is_file() {
        return Err(format!(
            "NOT_FOUND: section has no index.md {}",
            section_dir.display()
        ));
    }

    let mut markdown_files = Vec::new();
    collect_markdown_files(&section_dir, &mut markdown_files)?;
    build_rss_xml(public_dir, base_url, markdown_files)
}

fn build_rss_xml(public_dir: &Path, base_url: &str, markdown_files: Vec<PathBuf>) -> Result<String, String> {
    let mut items = Vec::new();
    for file_path in markdown_files {
        if file_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "index.md")
        {
            continue;
        }

        let raw_post = fs::read_to_string(&file_path).map_err(|e| {
            format!(
                "Failed to read markdown file {}: {}",
                file_path.display(),
                e
            )
        })?;

        let (headers, content) = render::get_headers(&raw_post);
        let frontmatter = if headers.trim().is_empty() {
            Frontmatter::default()
        } else {
            serde_yaml::from_str::<Frontmatter>(&headers).map_err(|e| {
                format!(
                    "Failed to parse frontmatter for {}: {}",
                    file_path.display(),
                    e
                )
            })?
        };

        if frontmatter.password.is_some() {
            continue;
        }

        let modified_time = fs::metadata(&file_path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let fallback_date = DateTime::<Utc>::from(modified_time).fixed_offset();
        let post_date = frontmatter
            .date
            .as_deref()
            .and_then(parse_post_date)
            .unwrap_or(fallback_date);

        let relative_path = file_path.strip_prefix(public_dir).unwrap_or(&file_path);
        let link = format!(
            "{}{}",
            normalize_base_url(base_url),
            markdown_path_to_url(relative_path)
        );

        let title = frontmatter
            .title
            .unwrap_or_else(|| title_from_path(relative_path));
        let description = frontmatter
            .description
            .unwrap_or_else(|| description_from_markdown(&content));

        items.push(FeedItem {
            title,
            link,
            description,
            pub_date: post_date.to_rfc2822(),
            timestamp: post_date.timestamp(),
        });
    }

    items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(render_rss_xml(base_url, &items))
}

fn collect_markdown_files(dir_path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Failed to read directory {}: {}", dir_path.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "Failed to read directory entry in {}: {}",
                dir_path.display(),
                e
            )
        })?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_markdown_files(&path, files)?;
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            files.push(path);
        }
    }

    Ok(())
}

fn parse_post_date(date: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(date)
        .ok()
        .or_else(|| DateTime::parse_from_rfc2822(date).ok())
}

fn markdown_path_to_url(path: &Path) -> String {
    let mut url_path = path.to_string_lossy().replace('\\', "/");
    if url_path.ends_with(".md") {
        let new_len = url_path.len() - 3;
        url_path.truncate(new_len);
    }

    if url_path.ends_with("/index") {
        let new_len = url_path.len() - 6;
        url_path.truncate(new_len);
    }

    if url_path.is_empty() {
        return "/".to_string();
    }

    if !url_path.starts_with('/') {
        url_path.insert(0, '/');
    }

    url_path
}

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(|name| name.replace(['-', '_'], " "))
        .unwrap_or_else(|| "Untitled".to_string())
}

fn description_from_markdown(content: &str) -> String {
    let first_non_empty_line = content
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();

    if first_non_empty_line.is_empty() {
        return "No description".to_string();
    }

    first_non_empty_line.chars().take(240).collect()
}

fn normalize_base_url(base_url: &str) -> String {
    base_url.trim_end_matches('/').to_string()
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn render_rss_xml(base_url: &str, items: &[FeedItem]) -> String {
    let normalized_base_url = normalize_base_url(base_url);
    let last_build_date = items
        .first()
        .map(|item| item.pub_date.as_str())
        .unwrap_or("Thu, 01 Jan 1970 00:00:00 +0000");

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n  <channel>\n",
    );
    xml.push_str("    <title>kpublish</title>\n");
    xml.push_str(&format!(
        "    <link>{}</link>\n",
        escape_xml(&normalized_base_url)
    ));
    xml.push_str("    <description>kpublish RSS feed</description>\n");
    xml.push_str(&format!(
        "    <lastBuildDate>{}</lastBuildDate>\n",
        escape_xml(last_build_date)
    ));

    for item in items {
        xml.push_str("    <item>\n");
        xml.push_str(&format!("      <title>{}</title>\n", escape_xml(&item.title)));
        xml.push_str(&format!("      <link>{}</link>\n", escape_xml(&item.link)));
        xml.push_str(&format!("      <guid>{}</guid>\n", escape_xml(&item.link)));
        xml.push_str(&format!(
            "      <description>{}</description>\n",
            escape_xml(&item.description)
        ));
        xml.push_str(&format!(
            "      <pubDate>{}</pubDate>\n",
            escape_xml(&item.pub_date)
        ));
        xml.push_str("    </item>\n");
    }

    xml.push_str("  </channel>\n</rss>\n");
    xml
}
