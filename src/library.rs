// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::href::Href;
use crate::{fnv1_hash::Hashable, md_content::MdContent};
use build_html as html;
use glob;
use html::{Html, HtmlContainer};
use ron;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error, ffi, fmt, fs, path::Path, rc::Rc, result};
use time;

/// Represents a library and holds information about its documents.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Library {
    /// A [`HashMap`] of file paths to documents and their doc info as stored in
    /// a [`Document`] struct.
    ///
    /// [`HashMap`]: HashMap
    /// [`Document`]: Document
    documents: HashMap<Rc<str>, Document>,
}

impl Library {
    /// Scans the current directory for any files ending in the ".md" file
    /// extension and creates a new [`Library`] by opening each file as a
    /// [`Document`].
    ///
    /// [`Document`]: Document
    /// [`Library`]: Library
    pub fn scan() -> Result<Self> {
        Ok(Self {
            documents: glob::glob("./**/*.md")?
                .filter_map(|path| {
                    let path = path.ok()?;
                    let doc = Document::open(&path).ok()?;
                    Some((path.as_os_str().to_str()?.into(), doc))
                })
                .collect(),
        })
    }

    /// Scans the current directory for markdown files and returns a [`Vec`] of
    /// documents not yet included in the [`Library`].
    ///
    /// [`Vec`]: Vec
    /// [`Library`]: Library
    pub fn scan_for_new(&self) -> Result<Vec<Rc<str>>> {
        Ok(glob::glob("./**/*.md")?
            .filter_map(|file| {
                let file = file.ok()?;
                let path = file.as_os_str().to_str()?;
                match self.documents.contains_key(path) {
                    true => None,
                    false => Some(path.into()),
                }
            })
            .collect())
    }

    /// Reads a serialized [`Library`] from a RON file with the given path.
    ///
    /// [`Library`]: Library
    #[inline]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        ron::from_str(
            fs::read_to_string(path)
                .map_err(|_| Error::FileReadError)?
                .as_str(),
        )
        .map_err(|_| Error::DeserializationError)
    }

    /// Saves the [`Library`], in RON format, to the given file path.
    ///
    /// [`Library`]: Library
    #[inline]
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(
            path,
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .map_err(|_| Error::SerializationError)?,
        )
        .map_err(|_| Error::FileWriteError)
    }

    /// Opens a [`Document`] at the given path and adds it to the [`Library`].
    ///
    /// [`Document`]: Document
    /// [`Library`]: Library
    pub fn add_document(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let doc = Document::open(&path)?;
        let path = match path.as_ref().as_os_str().to_str() {
            Some(s) => Ok(s.into()),
            None => Err(Error::InvalidString),
        }?;

        self.documents.insert(path, doc);
        Ok(())
    }

    /// Gets the backing hashmap of the [`Library`] which has value of type
    /// [`Document`] that are keyed with [`String`]s of the [`Document`]'s file
    /// path.
    ///
    /// [`Library`]: Library
    /// [`Document`]: Document
    /// [`String`]: String
    #[inline]
    #[must_use]
    pub fn documents(&self) -> &HashMap<Rc<str>, Document> {
        &self.documents
    }

    /// Updates all [`Document`] items within the [`Library`].
    //
    /// [`Document`]: Document
    /// [`Library`]: Library
    pub fn update(self) -> Result<Self> {
        Ok(Self {
            documents: self
                .documents
                .into_iter()
                .map(|(p, d)| -> Result<(Rc<str>, Document)> {
                    let s = &*p;
                    let doc = d.update(&s)?;
                    Ok((p, doc))
                })
                .filter_map(result::Result::ok)
                .collect(),
        })
    }

    /// Checks each of this [`Library`]'s documents for change since last update
    /// and returns a [`Vec`] containing the paths of those [`Document`]s. This
    /// function does not propagate I/O errors from reading documents.
    ///
    /// [`Library`]: Library
    /// [`Vec`]: Vec
    /// [`Document`]: Document
    pub fn changed_docs(&self) -> Vec<&str> {
        self.documents
            .iter()
            .filter_map(|(p, d)| match d.has_changed(&p.as_ref()).ok()? {
                true => Some(p.as_ref()),
                false => None,
            })
            .collect()
    }

    /// Creates a returns a [`LibraryHtml`] from documents managed by this
    /// [`Library`].
    ///
    /// [`Library`]: Library
    /// [`LibraryHtml`]: LibraryHtml
    pub fn gen_html(&self) -> Result<LibraryHtml> {
        let mut pages: Vec<(String, html::HtmlPage)> = self
            .documents
            .keys()
            .map(|p| -> Result<(String, html::HtmlPage)> {
                let href = p.replace(".md", ".html");
                let md = MdContent::new(
                    fs::read_to_string(&p.as_ref()).map_err(|_| Error::FileReadError)?,
                );

                let title = match md.title() {
                    Some(cow_str) => cow_str.as_ref().to_owned(),
                    None => "".to_owned(),
                };

                Ok((
                    href,
                    html::HtmlPage::new()
                        .with_title(title)
                        .with_link(
                            "../".to_owned().repeat(p.clone().path_items() - 1) + "index.html",
                            "HOME",
                        )
                        .with_html(md),
                ))
            })
            .filter_map(result::Result::ok)
            .collect::<Vec<_>>();

        if pages.len() != self.documents.len() {
            // At least one item was filtered out and an error must have occured.
            return Err(Error::FileReadError);
        }

        let list = self.documents.iter().fold(
            html::Container::new(html::ContainerType::UnorderedList),
            |acc, (p, d)| acc.with_link(p.replace(".md", ".html"), d.name()),
        );

        pages.push((
            "index.html".to_owned(),
            html::HtmlPage::new()
                .with_title("HOME")
                .with_header(1, "HOME")
                .with_container(list),
        ));

        Ok(LibraryHtml::new(pages))
    }
}

/// Contains the HTML representation of documents managed by a [`Library`] and
/// can write the library's HTML to disk.
#[derive(Debug)]
pub struct LibraryHtml {
    pages: Vec<(String, html::HtmlPage)>,
}

impl LibraryHtml {
    /// Creates a new [`LibraryHtml`] struct given a [`Vec`] of tuples in which
    /// the first item is a [`String`] holding the href path of the [`HtmlPage`]
    /// which is the tuple's second item.
    ///
    /// [`LibraryHtml`]: LibraryHtml
    /// [`Vec`]: Vec
    /// [`String`]: String
    /// [`HtmlPage`]: html::HtmlPage
    #[inline]
    #[must_use]
    pub fn new(pages: Vec<(String, html::HtmlPage)>) -> Self {
        Self { pages }
    }

    /// Consumes the given [`LibraryHtml`] and writes it to files, corrosponding
    /// with there href paths, to the given directory.
    ///
    /// [`LibraryHtml`]: LibraryHtml
    pub fn write(self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        for (href, page) in self.pages {
            let mut file_path = path.clone();
            file_path.push(href);

            if let Some(p) = file_path.parent() {
                fs::create_dir_all(p).map_err(|_| Error::DirectoryCreateError)?;
            }

            fs::write(file_path, page.to_html_string()).map_err(|_| Error::FileWriteError)?;
        }

        Ok(())
    }
}

/// Holds infomation about a markdown document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Document {
    name: Rc<str>,
    hash: u64,
    mod_time: time::OffsetDateTime,
}

impl Document {
    /// Opens the given path and reads it for info, this will set the
    /// modification time to the current time and as such should be avoided in
    /// favor of using methods of [`Library`].
    ///
    /// [`Library`]: Library
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let content = MdContent::new(fs::read_to_string(path).map_err(|_| Error::FileReadError)?);
        Ok(Self {
            name: match content.title() {
                Some(cow_str) => cow_str.as_ref().into(),
                None => "".into(),
            },
            hash: content.fnv1_hash(),
            mod_time: time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()),
        })
    }

    /// Updates the given [`Document`] by comparing its stored hash of the given
    /// file's content, if they are unequal then the modification time is
    /// updated to be the current time and the stored hash is updated.
    ///
    /// [`Document`]: Document
    /// [`MdContent`]: MdContent
    pub fn update(self, path: impl AsRef<Path>) -> Result<Self> {
        let content = MdContent::new(fs::read_to_string(path).map_err(|_| Error::FileReadError)?);
        let new_hash = content.fnv1_hash();

        Ok(match self.hash == new_hash {
            true => self,
            false => Self {
                name: match content.title() {
                    Some(cow_str) => cow_str.as_ref().into(),
                    None => "".into(),
                },
                hash: new_hash,
                mod_time: time::OffsetDateTime::now_local()
                    .unwrap_or(time::OffsetDateTime::now_utc()),
            },
        })
    }

    /// Returns true if the [`Document`] has changed since its last update. This
    /// is checked by taking the hash of the given file and comparing it to that
    /// which is stored within the [`Document`].
    ///
    /// [`Document`]: Document
    pub fn has_changed(&self, path: impl AsRef<Path>) -> Result<bool> {
        let content = MdContent::new(fs::read_to_string(path).map_err(|_| Error::FileReadError)?);
        Ok(self.hash != content.fnv1_hash())
    }

    /// Gets the time of the last modification as made by either the struct's
    /// construction or an update.
    #[inline]
    #[must_use]
    pub fn mod_time(&self) -> time::OffsetDateTime {
        self.mod_time
    }

    /// Gets a [`Cow<String>`] enclosing a reference to this [`Document`]'s name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Represents a result of some library related function.
pub type Result<T> = result::Result<T, Error>;

/// Represents a library error.
#[derive(Debug, Clone)]
pub enum Error {
    /// A [`glob`] [`PatternError`] error.
    ///
    /// [`glob`]: glob
    /// [`PatternError`]: glob::PatternError
    PatternError,

    /// The [`OsString`] was not valud UTF-8 nor could it be converted to a
    /// UTF-8 [`String.`]
    ///
    /// [`String`]: String
    /// [`OsString`]: ffi::OsString
    InvalidString,

    /// Could not deserialize a struct from given input.
    DeserializationError,

    /// I/O failure to read a directory.
    DirectoryReadError,

    /// I/O failure to create a directory.
    DirectoryCreateError,

    /// I/O failure to read file.
    FileReadError,

    /// I/O failure to write to file.
    FileWriteError,

    /// Failure to serialize the struct.
    SerializationError,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<glob::PatternError> for Error {
    fn from(_: glob::PatternError) -> Self {
        Self::PatternError
    }
}

// This look really weird but `OsString` is used as the error type for `Result`s
// returned by `OsString::into_string()`.
impl From<ffi::OsString> for Error {
    fn from(_: ffi::OsString) -> Self {
        Self::InvalidString
    }
}
