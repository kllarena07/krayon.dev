use pulldown_cmark::{Parser, html};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Post {
    filename: String,
    date: String,
    size: u64,
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_lowercase()
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap_or(""),
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp"
        )
    } else {
        false
    }
}

fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_str().unwrap_or(""), "mp4" | "mov")
    } else {
        false
    }
}

fn copy_assets(post_dir: &Path, output_base: &Path) {
    let assets_dir = output_base.join("assets");
    fs::create_dir_all(&assets_dir).expect("Failed to create assets directory");

    if let Ok(entries) = fs::read_dir(post_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && (is_image_file(&path) || is_video_file(&path))
                && path.file_name() != Some(std::ffi::OsStr::new("index.md"))
            {
                let filename = path.file_name().unwrap();
                let dest = assets_dir.join(filename);
                fs::copy(&path, &dest).expect("Failed to copy asset");
            }
        }
    }
}

fn update_asset_paths(html: &str, dirname: &str) -> String {
    html.replace("src=\"", &format!("src=\"./{}/assets/", dirname))
}

fn inject_video_fallback(html: &str) -> String {
    let re = Regex::new(r#"<video([^>]*)></video>"#).unwrap();
    re.replace_all(
        html,
        r#"<video$1><p>Your browser does not support the video tag.</p></video>"#,
    )
    .to_string()
}

fn main() {
    // Create blog directory if it doesn't exist
    fs::create_dir_all("blog").expect("Failed to create blog directory");

    let mut posts = Vec::new();

    // Process posts directories
    let posts_dir = Path::new("posts");
    if posts_dir.exists() {
        let entries = fs::read_dir(posts_dir).expect("Failed to read posts directory");
        for entry in entries {
            let entry = entry.expect("Failed to read entry");
            let dir_path = entry.path();
            if dir_path.is_dir() {
                let dirname = dir_path.file_name().unwrap().to_str().unwrap();

                // Validate dirname contains only letters, numbers, and spaces
                if !dirname
                    .chars()
                    .all(|c| c.is_alphabetic() || c.is_numeric() || c == ' ')
                {
                    panic!(
                        "Directory name '{}' contains invalid characters. Only letters (a-z, A-Z), numbers (0-9), and spaces are allowed.",
                        dirname
                    );
                }

                // Check for index.md
                let index_path = dir_path.join("index.md");
                if !index_path.exists() {
                    panic!("Missing index.md in directory '{}'", dirname);
                }

                let content = fs::read_to_string(&index_path).expect("Failed to read index.md");
                let mut html = markdown_to_html(&content);

                let sanitized = sanitize_filename(dirname);

                // Copy assets
                copy_assets(&dir_path, Path::new(&format!("blog/{}", sanitized)));

                // Update asset paths
                html = update_asset_paths(&html, &sanitized);
                html = inject_video_fallback(&html);

                let output_path = format!("blog/{}.html", sanitized);
                fs::write(&output_path, html).expect("Failed to write HTML file");
                println!("cargo:warning=Generated: {}", output_path);

                let metadata = fs::metadata(&output_path).expect("Failed to get metadata");
                let size = metadata.len();

                // Use post directory's modified time for date
                let dir_metadata =
                    fs::metadata(&dir_path).expect("Failed to get directory metadata");
                let modified = dir_metadata
                    .modified()
                    .expect("Failed to get modified time");
                let datetime: chrono::DateTime<chrono::Utc> = modified.into();
                let date = datetime.format("%d-%b-%Y %H:%M").to_string();

                if dirname != "index" {
                    posts.push(Post {
                        filename: format!("{}.html", sanitized),
                        date,
                        size,
                    });
                }
            }
        }
    }

    // Ensure index.html was generated
    if !std::path::Path::new("blog/index.html").exists() {
        panic!("Failed to generate blog/index.html. Ensure posts/index/ and index.md exist.");
    }

    // Write the posts to a JSON file
    let json = serde_json::to_string(&posts).expect("Failed to serialize posts");
    fs::write("blog/posts.json", json).expect("Failed to write posts.json");
}
