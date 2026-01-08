use axum::{
    Router,
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Post {
    filename: String,
    date: String,
    size: u64,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticFiles;

#[derive(RustEmbed)]
#[folder = "blog/"]
struct BlogFiles;

async fn index() -> Response {
    let content_file = BlogFiles::get("index.html").unwrap();
    let content = String::from_utf8(content_file.data.into_owned()).unwrap();
    let template_file = StaticFiles::get("blog_file.html").unwrap();
    let template_str = String::from_utf8(template_file.data.into_owned()).unwrap();
    let full_content = template_str
        .replace("{{filename}}", "deploying-a-portfolio-over-ssh.html")
        .replace("{{content}}", &content);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(full_content))
        .unwrap()
}

async fn blog_file(Path(filename): Path<String>) -> Response {
    if let Some(content_file) = BlogFiles::get(&filename) {
        if filename.ends_with(".html") {
            let content = String::from_utf8(content_file.data.into_owned()).unwrap();
            let template = StaticFiles::get("blog_file.html").unwrap();
            let template_str = String::from_utf8(template.data.into_owned()).unwrap();
            let full_content = template_str
                .replace("{{filename}}", &filename)
                .replace("{{content}}", &content);
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(full_content))
                .unwrap()
        } else {
            let data = content_file.data.into_owned();
            let content_type = mime_guess::from_path(&filename)
                .first_or_octet_stream()
                .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header("Cache-Control", "public, max-age=31536000")
                .body(axum::body::Body::from(data))
                .unwrap()
        }
    } else {
        not_found().await
    }
}

async fn blog_index() -> Response {
    let template_file = StaticFiles::get("blog.html").unwrap();
    let template = String::from_utf8(template_file.data.into_owned()).unwrap();

    let posts_json = BlogFiles::get("posts.json");
    let posts: Vec<Post> = if let Some(pj) = posts_json {
        serde_json::from_slice(&pj.data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let row_template = StaticFiles::get("blog_row.html").unwrap();
    let row_template_str = String::from_utf8(row_template.data.into_owned()).unwrap();

    let mut links = String::new();
    for post in &posts {
        let row = row_template_str
            .replace("{{filename}}", &post.filename)
            .replace("{{date}}", &post.date)
            .replace("{{size}}", &post.size.to_string());
        links.push_str(&row);
    }

    let index_html = template.replace("{{content}}", &links);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(index_html))
        .unwrap()
}

async fn not_found() -> Response {
    let file = StaticFiles::get("404.html").unwrap();
    let content = String::from_utf8(file.data.into_owned()).unwrap();
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(content))
        .unwrap()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://localhost:3000");

    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog_index))
        .route("/blog/", get(blog_index))
        .route("/blog/{*filename}", get(blog_file))
        .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
