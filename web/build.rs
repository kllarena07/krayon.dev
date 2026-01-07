use pulldown_cmark::{Parser, html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Post {
    filename: String,
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

fn main() {
    // Create blog directory if it doesn't exist
    fs::create_dir_all("blog").expect("Failed to create blog directory");

    let mut posts = Vec::new();

    // Process markdown files
    let markdown_dir = Path::new("markdown");
    if markdown_dir.exists() {
        let entries = fs::read_dir(markdown_dir).expect("Failed to read markdown directory");
        for entry in entries {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("md")) {
                let content = fs::read_to_string(&path).expect("Failed to read markdown file");
                let html = markdown_to_html(&content);
                let filename = path.file_stem().unwrap().to_str().unwrap();
                let sanitized = sanitize_filename(filename);
                let output_path = format!("blog/{}.html", sanitized);
                fs::write(&output_path, html).expect("Failed to write HTML file");
                println!("cargo:warning=Generated: {}", output_path);

                posts.push(Post {
                    filename: format!("{}.html", sanitized),
                });
            }
        }
    }

    // Write the posts to a JSON file
    let json = serde_json::to_string(&posts).expect("Failed to serialize posts");
    fs::write("blog/posts.json", json).expect("Failed to write posts.json");
}
