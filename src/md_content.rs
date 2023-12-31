// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::fnv1_hash::Hashable;
use build_html as html;
use pulldown_cmark as md;
use std::{borrow::Cow, rc::Rc};

#[derive(Debug, Clone)]
pub struct MdContent {
    md_string: Rc<str>,
}

/// Represents a peice of markdown content.
impl MdContent {
    /// Creates a new [`MdContent`] given a markdown string.
    ///
    /// [`MdContent`]: MdContent
    #[inline]
    #[must_use]
    pub fn new(md_string: impl AsRef<str>) -> Self {
        Self {
            md_string: md_string.as_ref().into(),
        }
    }

    /// Gets a title from the [`MdContent`]. This looks for the first
    /// [`Heading`] with a level of [`H1`] and then returns the first found
    /// [`Text`] after that [`Heading`].
    ///
    /// [`MdDocument`]: MdDocument
    /// [`Heading`]: md::Tag::Heading
    /// [`H1`]: md::HeadingLevel::H1
    /// [`Text`]: md::Event::Text
    #[must_use]
    pub fn title(&self) -> Option<md::CowStr> {
        let mut parser = md::Parser::new(&self.md_string);

        while let Some(event) = parser.next() {
            match event {
                // Finds the first H1 heading in the document, if it exists.
                md::Event::Start(md::Tag::Heading(md::HeadingLevel::H1, _, _)) => {
                    for e in parser.by_ref() {
                        match e {
                            // Return first text found after the first found H1
                            // heading.
                            md::Event::Text(cs) => return Some(cs),
                            _ => continue,
                        }
                    }

                    // Already looped from first H1 heading to end, no need to
                    // continue the loop.
                    break;
                }

                _ => continue,
            }
        }

        None
    }
}

impl html::Html for MdContent {
    fn to_html_string(&self) -> String {
        let parser = md::Parser::new_ext(&self.md_string, md::Options::all());
        let mut html_string = String::new();
        md::html::push_html(&mut html_string, parser);
        html_string
    }
}

impl Hashable for MdContent {
    fn fnv1_hash(&self) -> u64 {
        self.md_string.as_bytes().fnv1_hash()
    }
}
