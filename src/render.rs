use handlebars::Handlebars;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Parser, Options};
use serde_json::json;
use actix_web::web;
use serde::Deserialize;
use serde_yaml;

#[derive(Deserialize)]
struct Headers {  
    title: Option<String>,
    template: Option<String>
}

//Я люблю сосать член
pub fn render(hb: web::Data<Handlebars<'_>>, name: &str, markdown: &str) -> String {
    let (headers, mark) = get_headers(markdown);
    let headers: Headers = serde_yaml::from_str(&headers).unwrap();

    let title = headers.title.unwrap_or(name.into());
    let template = headers.template.unwrap_or("post".to_owned());

    let data = json!({
        "name": title.clone(),
        "content": &mark_to_html(format!("# {}\n{}", title, mark).as_str())
    });
    let body = hb.render(&template, &data).unwrap();

    body
}



fn get_headers<R: AsRef<str>>(markdown: R) -> (String, String) {
    let (mut headers, mut data, mut started, mut finished) = (vec![], vec![], false, false);

    for line in markdown.as_ref().lines() {
        match [line == "---", started, finished] {
            [true, true, false] => finished = true,
            [true, false, false] => started = true,
            [false, true, false] => headers.push(line),
            _ => data.push(line),
        }
    }

    (headers.join("\n"), data.join("\n"))
}

fn mark_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(&markdown, Options::all());
    let mut buffer = String::new();
    push_html(&mut buffer, parser);
    buffer
}