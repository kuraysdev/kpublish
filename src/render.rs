use handlebars::Handlebars;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Parser, Options};
use serde_json::json;
use actix_web::{web};
use fronma::parser::parse;
use serde::Deserialize;

#[derive(Deserialize)]
struct Headers {
  title: String,
}

//Я люблю сосать член
pub fn render(hb: web::Data<Handlebars<'_>>, name: &str, markdown: &str) -> String {
    let md = parse::<Headers>(markdown).unwrap();
    let data = json!({
        "name": md.headers.title || name,
        "content": &mark_to_html(format!("# {}\n{}", name, markdown).as_str())
    });
    let body = hb.render("post", &data).unwrap();

    return body;
}



fn mark_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(&markdown, Options::all());
    let mut buffer = String::new();
    push_html(&mut buffer, parser);
    buffer
}