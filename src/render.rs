use horrorshow::Raw;
use horrorshow::html;
use horrorshow::helper::doctype;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Parser, Options};

//Я люблю сосать член
pub fn html(name: &str, markdown: &str) -> String {
    format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                : Raw(&make_header(name));
                body(class="markdown-body") {
                    : Raw(&mark_to_html(format!("# {}\n{}", name, markdown).as_str()));
                    script {
                        : Raw("hljs.initHighlighting()")
                    }
                    footer {
                        p { : "Author: Egor Abramov" }
                        p { a(href="mailto:kurays@kurays.ml") { : "kurays@kurays.ml" } }
                    }
                }
            }
        }
    )
}

fn mark_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(&markdown, Options::all());
    let mut buffer = String::new();
    push_html(&mut buffer, parser);
    buffer
}


fn make_header(name: &str) -> String {
    format!("{}",
        html! {
            head {
                link(rel="stylesheet", href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.2.0/github-markdown-dark.min.css");
                link(rel="stylesheet", href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/github-dark.min.css");
                script(src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js");
                meta(charset="utf-8");
                meta(property="og:title", content=&name);
                meta(property="og:type", content="website");
                meta(property="og:url", content="http://dev.kurays.ml:8080");
                meta(property="og:description", content="Потом сделаю");
                title { : &name }
                meta(name="viewport", content="width=device-width, initial-scale=1.0");
                style {
                    : "body { width: 80%; margin: 1% auto !important; }";
                    : "img { max-width: 80% }";
                    : "footer { position: absolute; bottom: 0;}"
                }
            }
        }
    )
}
