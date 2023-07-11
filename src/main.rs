use actix_web::{App, get, HttpServer, web, Responder, HttpResponse, HttpRequest};
use std::fs;
use std::fs::File;
use std::io::Read;
use actix_web::middleware::Logger;
use env_logger::Env;

mod render;
mod fileutil;


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("<h1>Hello kpublish!</h1>")

}

#[get("/md")]
async fn pages(req: HttpRequest) -> impl Responder {
    match fs::read_dir("public") {
        Ok(entries) =>  {
            let links: Vec<String> = entries
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let file_name = entry.file_name()
                        .to_string_lossy()
                        .into_owned()
                        .replace(".md", "");
                        let file_path = format!("public/{}.md", file_name);
                        let first_line = fileutil::get_first_line(&file_path);
                        Some(format!(
                            "[{}]({}) - {}",
                            file_name,
                            format!("{}/{}", req.uri(), file_name.replace(" ", "%20")),
                            first_line.unwrap_or("none".to_string()).replace("# ", "")
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            let combined_links = links.join("\n\n");
            HttpResponse::Ok().body(render::html(&combined_links))
        },
        Err(_) => HttpResponse::InternalServerError().body("Failed to read directory.")
    }
}


#[get("/md/{md}")]
async fn render_md_file(path: web::Path<String>) -> HttpResponse {
    let md = path.into_inner();
    match File::open(format!("public/{}.md", md)) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let html_output = render::html(&contents);
            HttpResponse::Ok().body(html_output)
        },
        Err(_) => HttpResponse::InternalServerError().body(render::html("# НЕт ТаКоГо ФАЙла МЛЯТЬ. \n\n Создать хочешь? \n\n ПЕРЕХОЧЕШЬ"))
    }
    



    
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
        .wrap(Logger::default())
        .wrap(Logger::new("%a %{User-Agent}i"))
        .service(index)
        .service(render_md_file)
        .service(pages)
    }).bind(("0.0.0.0", 8080))?
    .run().await?;
    Ok(())
}
