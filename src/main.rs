use actix_web::web::Data;
use actix_web::{get, web, App, HttpResponse, HttpServer, HttpRequest};
use actix_files::NamedFile;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use actix_web::middleware::Logger;
use env_logger::Env;
use handlebars::Handlebars;

mod render;
mod fileutil;





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
    println!("{}", post);
    let requested_path = Path::new("public").join(format!("{}", post));

    let file_path = if requested_path.is_dir() || post.is_empty() {
        requested_path.join("index.md")
    } else if requested_path.extension().is_none() {
        requested_path.with_extension("md")
    } else {
        requested_path
    };

    if !file_path.exists() || !file_path.is_file() {
        // Если файл не найден, возвращаем ошибку 404
        return HttpResponse::NotFound().body("404").into();
    }

    if file_path.extension().unwrap() == "md" {
        // Если md файл рендерим его
        let mut contents = String::new();
        let mut file = File::open(file_path).unwrap();
        file.read_to_string(&mut contents).unwrap();
        let html_output = render::render(hb, &post, &contents);
        HttpResponse::Ok().body(html_output)
    } else  {
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
        //.service(index)
        .service(file_tree)
        .service(return_file)
        //.service(pages)
    }).bind(("0.0.0.0", 8080))?
    .run().await?;
    Ok(())
}
