use actix_web::{App, HttpServer, web, Responder};

async fn index() -> impl Responder {
    "Hello world!"
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");
    HttpServer::new(|| {
        App::new().service(
            // prefixes all resources and routes attached to it...
            web::scope("/app")
                // ...so this handles requests for `GET /app/index.html`
                .route("/index.html", web::get().to(index)),
        )
    }).bind(("127.0.0.1", 8080))?
    .run().await?;
    Ok(())
}
