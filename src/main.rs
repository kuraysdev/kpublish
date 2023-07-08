use actix_web::{App, get, HttpServer, web, Responder, HttpResponse};
use actix_files as fs;
use std::fs::File;
use std::io::Read;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Options, Parser};


#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("<h1>Hello world!</h1>")

}
#[get("/md/{md}")]
async fn render_md_file(path: web::Path<String>) -> HttpResponse {
    let md = path.into_inner();
    let mut file = File::open(format!("public/{}.md", md)).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let parser = Parser::new_ext(&contents, Options::all());
    let mut html_output = String::new();
    push_html(&mut html_output, parser);

    HttpResponse::Ok().body(html_output)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    HttpServer::new(|| {
        App::new()
        .service(index)
        //.service(fs::Files::new("/md", "public").show_files_listing())
        .service(render_md_file)
    }).bind(("127.0.0.1", 8080))?
    .run().await?;
    Ok(())
}
