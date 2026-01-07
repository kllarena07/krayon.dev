use axum::{Router, extract::Path, http::StatusCode, response::Html, routing::get};
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

async fn index() -> Html<String> {
    let file = StaticFiles::get("index.html").unwrap();
    let content = String::from_utf8(file.data.into_owned()).unwrap();
    Html(content)
}

async fn blog_file(Path(filename): Path<String>) -> (StatusCode, Html<String>) {
    if let Some(content_file) = BlogFiles::get(&filename) {
        let content = String::from_utf8(content_file.data.into_owned()).unwrap();
        (StatusCode::OK, Html(content))
    } else {
        not_found().await
    }
}

async fn blog_index() -> Html<String> {
    let template_file = StaticFiles::get("blog.html").unwrap();
    let template = String::from_utf8(template_file.data.into_owned()).unwrap();

    let posts_json = BlogFiles::get("posts.json").unwrap();
    let posts: Vec<Post> = serde_json::from_slice(&posts_json.data).unwrap();

    let mut links = String::new();
    for post in &posts {
        let link = format!("<a href=\"/blog/{}\">{}</a>", post.filename, post.filename);
        links.push_str(&format!(
            r#"<tr>
                <td style="width: 398px;">{}</td>
                <td style="width: 200px;">{}</td>
                <td style="width: 8px;">{}</td>
            </tr>
"#,
            link, post.date, post.size
        ));
    }

    let index_html = template.replace("{{content}}", &links);

    Html(index_html)
}

async fn not_found() -> (StatusCode, Html<String>) {
    let file = StaticFiles::get("404.html").unwrap();
    let content = String::from_utf8(file.data.into_owned()).unwrap();
    (StatusCode::NOT_FOUND, Html(content))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://localhost:3000");

    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog_index))
        .route("/blog/", get(blog_index))
        .route("/blog/{filename}", get(blog_file))
        .fallback(not_found);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
