// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

mod md_content;

use build_html as html;
use html::{Html, HtmlContainer};

fn main() {
    let md = "# Some Title\nthis is a bunch of markdown\n - one\n - two\n - three\n".to_string();
    let page_content = md_content::MdContent::new(md);
    let page_title = page_content.title().unwrap_or_default();

    let content = html::Container::new(html::ContainerType::Div)
        .with_attributes(vec![("class", "content")])
        .with_html(page_content);

    let html_document = html::HtmlPage::new()
        .with_title(page_title)
        .with_container(content);

    println!("{}", html_document.to_html_string());
}
