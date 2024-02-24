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
    let md = parse::<Headers>(markdown).unwrap_or(parse::<>(markdown));
    let title: Option<String> = Some(md.headers.title);
    let titl = Some(title).unwrap_or(Some(name.to_owned())).unwrap();
    let data = json!({
        "name": titl,
        "content": &mark_to_html(format!("# {}\n{}", titl.as_str(), md.body).as_str())
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