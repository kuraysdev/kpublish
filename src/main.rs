use actix_web::web::Data;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, HttpRequest};
use actix_files::NamedFile;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use actix_web::middleware::Logger;
use env_logger::Env;
use handlebars::Handlebars;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;

mod render;
mod fileutil;

const POSTING_KEY: &str = "your-secure-key-here";


#[get("/admin")]
async fn admin_page(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let html = hb.render("admin", &json!({})).unwrap();
    HttpResponse::Ok().body(html)
}

#[get("/api/files/{path:.*}")]
async fn get_file(path: web::Path<String>) -> HttpResponse {
    let file_path = Path::new("public").join(path.into_inner());
    
    match fs::read_to_string(&file_path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::NotFound().body("File not found")
    }
}

#[post("/api/files/{path:.*}")]
async fn post_file(
    req: HttpRequest,
    path: web::Path<String>,
    body: String
) -> HttpResponse {
    // Check posting key
    match req.headers().get("X-Posting-Key") {
        Some(key) if key == POSTING_KEY => (),
        _ => return HttpResponse::Unauthorized().body("Invalid posting key")
    }

    let path_str = path.into_inner();
    let file_path = PathBuf::from("public").join(&path_str);

    // Create directories if they don't exist
    if let Some(parent) = file_path.parent() {
        if let Err(e) = create_dir_all(parent) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create directories: {}", e));
        }
    }

    // Write the file
    match fs::write(&file_path, body) {
        Ok(_) => HttpResponse::Ok().body("File saved successfully"),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to write file: {}", e))
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


#[get("/{post:.*}")]
async fn return_file(req: HttpRequest, path: web::Path<String>, hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let post = path.into_inner();
    let requested_path = Path::new("public").join(format!("{}", post));

    let file_path = if requested_path.is_dir() || post.is_empty() {
        requested_path.join("index.md")
    } else if requested_path.extension().is_none() {
        requested_path.with_extension("md")
    } else {
        requested_path
    };

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
            data = json!({
                "index": dir_index
            });
        }

        let html_output = render::render(hb, &post, &contents, Some(data));
        HttpResponse::Ok().body(html_output)
    } else {
        NamedFile::open(file_path).unwrap().into_response(&req)
    }
}

fn register_templates() -> Data<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_templates_directory(".html", "./templates").unwrap();
    println!("Registered {} templates", handlebars.get_templates().len().to_owned());
    web::Data::new(handlebars)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, kpublish!");
    let handlebars = register_templates();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        .wrap(Logger::new("%a %{User-Agent}i"))
        .app_data(handlebars.clone())
        .service(admin_page)
        .service(get_file)
        .service(post_file)
        .service(file_tree)
        .service(return_file)
    }).bind(("0.0.0.0", 8080))?
    .run().await?;
    Ok(())
}
