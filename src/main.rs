use actix_files::NamedFile;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer};
use env_logger::Env;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_yaml;
use std::collections::HashMap;
use std::fs::File;
use std::fs::{self, create_dir_all};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use tokio::sync::RwLock;

mod fileutil;
mod render;
mod rss;

#[derive(Deserialize)]
struct PostParams {
    md: Option<bool>,
    password: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Config {
    host: String,
    password: String,
    base_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Headers {
    title: Option<String>,
    template: Option<String>,
    data: Option<Value>,
    password: Option<String>,
}

struct AppState {
    rss_cache: RwLock<HashMap<String, String>>,
}

fn public_path(path: &str) -> Option<PathBuf> {
    let mut result = PathBuf::from("public");
    for part in Path::new(path).components() {
        if !matches!(part, std::path::Component::Normal(_)) { return None; }
        result.push(part.as_os_str());
        if fs::symlink_metadata(&result).map(|m| m.file_type().is_symlink()).unwrap_or(false) { return None; }
    }
    Some(result)
}

#[get("/admin")]
async fn admin_page(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let html = hb.render("admin", &json!({})).unwrap();
    HttpResponse::Ok().body(html)
}

#[get("/api/files/{path:.*}")]
async fn get_file(path: web::Path<String>) -> HttpResponse {
    let file_path = match public_path(&path.into_inner()) { Some(p) => p, None => return HttpResponse::BadRequest().body("Invalid path") };

    match fs::read_to_string(&file_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::NotFound().body("File not found"),
    }
}

#[post("/api/files/{path:.*}")]
async fn post_file(
    req: HttpRequest,
    path: web::Path<String>,
    config: web::Data<Config>,
    state: web::Data<AppState>,
    body: String,
) -> HttpResponse {
    // Check posting key
    match req.headers().get("X-Posting-Key") {
        Some(key) if *key == *config.password => (),
        _ => return HttpResponse::Unauthorized().body("Invalid posting key"),
    }

    let path_str = path.into_inner();
    let file_path = match public_path(&path_str) { Some(p) => p, None => return HttpResponse::BadRequest().body("Invalid path") };

    // Create directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if let Err(e) = create_dir_all(parent) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create directories: {}", e));
        }
    }

    // Write the file
    match fs::write(&file_path, body) {
        Ok(_) => {
            let mut rss_cache = state.rss_cache.write().await;
            rss_cache.clear();
            HttpResponse::Ok().body("File saved successfully")
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to write file: {}", e)),
    }
}

// Handler function to return JSON file tree
#[get("/filetree")]
async fn file_tree() -> HttpResponse {
    let public_folder_path = "./public"; // Change this to your public folder path
    let path = Path::new(public_folder_path);

    let file_tree = fileutil::traverse_folder(path);
    let json_response = serde_json::to_string(&file_tree).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response)
}

#[get("/rss.xml")]
async fn rss_feed(config: web::Data<Config>, state: web::Data<AppState>) -> HttpResponse {
    build_rss_response("", &config, &state).await
}

#[get("/{feed_path:.*}/rss.xml")]
async fn rss_feed_for_path(
    path: web::Path<String>,
    config: web::Data<Config>,
    state: web::Data<AppState>,
) -> HttpResponse {
    build_rss_response(path.as_str(), &config, &state).await
}

async fn build_rss_response(
    feed_path: &str,
    config: &web::Data<Config>,
    state: &web::Data<AppState>,
) -> HttpResponse {
    let normalized_feed_path = feed_path.trim_matches('/').to_owned();

    {
        let rss_cache = state.rss_cache.read().await;
        if let Some(xml) = rss_cache.get(&normalized_feed_path) {
            return HttpResponse::Ok()
                //.append_header(("Content-Type", "application/rss+xml; charset=utf-8"))
                .body(xml.clone());
        }
    }

    match rss::build_rss_feed("public", &config.base_url, &normalized_feed_path) {
        Ok(xml) => {
            let mut rss_cache = state.rss_cache.write().await;
            rss_cache.insert(normalized_feed_path, xml.clone());
            HttpResponse::Ok()
                //.append_header(("Content-Type", "application/rss+xml; charset=utf-8"))
                .body(xml)
        }
        Err(e) if e.starts_with("NOT_FOUND:") => HttpResponse::NotFound().body("404"),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[get("/{post:.*}")]
async fn return_file(
    req: HttpRequest,
    path: web::Path<String>,
    params: web::Query<PostParams>,
    hb: web::Data<Handlebars<'_>>,
) -> HttpResponse {
    let post = path.into_inner();
    let requested_path = match public_path(&post) { Some(p) => p, None => return HttpResponse::BadRequest().body("Invalid path") };

    let file_path = if requested_path.is_dir() || post.is_empty() {
        requested_path.join("index.md")
    } else if requested_path.extension().is_none() {
        requested_path.with_extension("md")
    } else {
        requested_path
    };

    if public_path(&file_path.strip_prefix("public").unwrap().to_string_lossy()).is_none() { return HttpResponse::BadRequest().body("Invalid path"); }
    if !file_path.exists() || !file_path.is_file() {
        return HttpResponse::NotFound().body("404");
    }

    if file_path.extension().unwrap() == "md" {
        // Read the markdown file
        let mut contents = String::new();
        let mut file = File::open(&file_path).unwrap();
        file.read_to_string(&mut contents).unwrap();

        // If it's an index file, get directory contents
        let mut data = json!({});
        if file_path.file_name().unwrap() == "index.md" {
            let dir_index = fileutil::get_directory_index(file_path.parent().unwrap());
            println!("index: {:?}", dir_index);
            data = json!({
                "index": dir_index
            });
        }

        let (headers, md) = render::get_headers(&contents);
        let headers: Headers = serde_yaml::from_str(&headers).unwrap();

        if headers.password.as_ref() != params.password.as_ref() {
            return HttpResponse::Forbidden()
                .append_header(("Content-Type", "text/plain; charset=utf-8"))
                .body("No!");
        }

        if params.md.unwrap_or(false) {
            return HttpResponse::Ok()
                .append_header(("Content-Type", "text/plain; charset=utf-8"))
                .body(contents);
        }

        let html_output = render::render(hb, &post, &contents, Some(data));
        HttpResponse::Ok().body(html_output)
    } else {
        NamedFile::open(file_path).unwrap().into_response(&req)
    }
}

fn register_templates() -> Data<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./templates")
        .unwrap();
    println!(
        "Registered {} templates",
        handlebars.get_templates().len().to_owned()
    );
    web::Data::new(handlebars)
}

fn load_config() -> Config {
    let yaml_str = fs::read_to_string("config.yaml").expect("Не удалось прочитать config.yaml");

    serde_yaml::from_str(&yaml_str).expect("Ошибка парсинга YAML")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, kpublish!");
    let handlebars = register_templates();
    let config = load_config();

    let config_data = web::Data::new(config.clone());
    let state_data = web::Data::new(AppState {
        rss_cache: RwLock::new(HashMap::new()),
    });

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(handlebars.clone())
            .app_data(config_data.clone())
            .app_data(state_data.clone())
            .service(admin_page)
            .service(get_file)
            .service(post_file)
            .service(file_tree)
            .service(rss_feed)
            .service(rss_feed_for_path)
            .service(return_file)
    })
    .bind(config.host)?
    .run()
    .await?;
    Ok(())
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    #[test]
    fn rejects_parent_and_absolute_paths() {
        assert!(public_path("../config.yaml").is_none());
        assert!(public_path("blog/../../config.yaml").is_none());
        assert!(public_path("/etc/passwd").is_none());
        assert_eq!(public_path("blog/article.md"), Some(PathBuf::from("public/blog/article.md")));
    }
}
