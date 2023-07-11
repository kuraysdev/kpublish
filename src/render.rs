use horrorshow::Raw;
use horrorshow::html;
use horrorshow::helper::doctype;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Parser, Options};

//Я люблю сосать член
pub fn html(markdown: &str) -> String {
    format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    link(rel="stylesheet", href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.2.0/github-markdown-dark.min.css") {}
                    link(rel="stylesheet", href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/github-dark.min.css") {}
                    script(src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js") {}
                    meta(charset="utf-8"){}
                    meta(name="viewport", content="width=device-width, initial-scale=1.0"){}
                    style {
                        : "body { width: 80%; margin: 1% auto !important }";
                        : "img { max-width: 80% }"
                    }
                }
                body(class="markdown-body") {
                    : Raw(&mark_to_html(markdown));
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
