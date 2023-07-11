// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use build_html::{self, Html, HtmlContainer, HtmlPage};
use pulldown_cmark::{self, Parser};

fn main() {
    let md = "# Some Title\nthis is a bunch or markdown\n - one\n - two\n - three\n";
    let parser = Parser::new(md);
    let mut parsed = String::new();
    let title = match first_text(parser.into_iter()) {
        Some(s) => s,
        None => String::from(""),
    };
    let parser = Parser::new(md);
    pulldown_cmark::html::push_html(&mut parsed, parser);

    let mut html_document = HtmlPage::new().with_title(title);
    html_document.add_raw(&mut parsed);
    println!("{}", html_document.to_html_string());
}

fn first_text(parser: Parser) -> Option<String> {
    match parser
        .into_iter()
        .map(|e| match e {
            pulldown_cmark::Event::Text(cs) => Some(cs),
            _ => None,
        })
        .filter(|e| e.is_some())
        .next()
    {
        Some(t) => match t.unwrap() {
            pulldown_cmark::CowStr::Boxed(s) => Some(String::from(s)),
            pulldown_cmark::CowStr::Borrowed(s) => Some(String::from(s)),
            pulldown_cmark::CowStr::Inlined(s) => Some(String::from(&*s)),
        },
        None => None,
    }
}
