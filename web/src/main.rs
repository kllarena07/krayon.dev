use actix_web::{App, HttpResponse, HttpServer, Result, web};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Post {
    filename: String,
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticFiles;

#[derive(RustEmbed)]
#[folder = "blog/"]
struct BlogFiles;

async fn index() -> Result<HttpResponse> {
    let file = StaticFiles::get("index.html").unwrap();
    let content = String::from_utf8(file.data.into_owned()).unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(content))
}

async fn serve_blog_file(path: web::Path<String>) -> Result<HttpResponse> {
    let filename = path.into_inner();
    if let Some(content_file) = BlogFiles::get(&filename) {
        //let template_file = StaticFiles::get("blog.html").unwrap();
        //let template = String::from_utf8(template_file.data.into_owned()).unwrap();
        let content = String::from_utf8(content_file.data.into_owned()).unwrap();
        //let hydrated = template.replace("{{content}}", &content);
        Ok(HttpResponse::Ok().content_type("text/html").body(content))
    } else {
        not_found().await
    }
}

async fn serve_blog_index() -> Result<HttpResponse> {
    let template_file = StaticFiles::get("blog.html").unwrap();
    let template = String::from_utf8(template_file.data.into_owned()).unwrap();

    let posts_json = BlogFiles::get("posts.json").unwrap();
    let posts: Vec<Post> = serde_json::from_slice(&posts_json.data).unwrap();

    let mut links = String::new();
    for post in posts {
        links.push_str(&format!(
            "\n<a href=\"/blog/{}\">{}</a>",
            post.filename, post.filename
        ));
    }

    let index_html = template.replace("{{content}}", &links);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index_html))
}

async fn not_found() -> Result<HttpResponse> {
    let file = StaticFiles::get("404.html").unwrap();
    let content = String::from_utf8(file.data.into_owned()).unwrap();
    Ok(HttpResponse::NotFound()
        .content_type("text/html")
        .body(content))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://localhost:3000");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/blog", web::get().to(serve_blog_index))
            .route("/blog/", web::get().to(serve_blog_index))
            .route("/blog/{filename}", web::get().to(serve_blog_file))
            .default_service(web::route().to(not_found))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

