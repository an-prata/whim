// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::{fnv1_hash::Hashable, md_content::MdContent};
use glob;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::{collections::HashMap, fmt, fs, path::Path, result};
use std::{error, ffi};
use time;
use toml;

/// Represents a library and holds information about its documents.
#[derive(Clone, Serialize, Deserialize)]
pub struct Library {
    /// A [`HashMap`] of file paths to documents and their doc info as stored in
    /// a [`Document`] struct.
    ///
    /// [`HashMap`]: HashMap
    /// [`Document`]: Document
    documents: HashMap<String, Document>,
}

impl Library {
    /// Creates a new, empty [`Library`].
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Scans the current directory for any files ending in the ".md" file
    /// extension and creates a new [`Library`] by opening each file as a
    /// [`Document`].
    ///
    /// [`Document`]: Document
    /// [`Library`]: Library
    #[must_use]
    pub fn scan() -> Result<Self> {
        Ok(Self {
            documents: glob::glob("**.md")?
                .filter_map(result::Result::ok)
                .map(|path| -> Result<(String, Document)> {
                    let doc = Document::open(&path)?;
                    Ok((path.into_os_string().into_string()?, doc))
                })
                .filter_map(result::Result::ok)
                .collect(),
        })
    }

    /// Reads a serialized [`Library`] from a file with the given path.
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(toml::from_str(
            fs::read_to_string(path)
                .map_err(|_| Error::FileReadError)?
                .as_str(),
        )?)
    }

    /// Saves the [`Library`], in TOML format, to the given file path.
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::write(path, toml::to_string(self)?).map_err(|_| Error::FileWriteError)
    }

    /// Opens a [`Document`] at the given path and adds it to the [`Library`].
    ///
    /// [`Document`]: Document
    /// [`Library`]: Library
    pub fn add_doc(&mut self, path: String) -> Result<()> {
        let doc = Document::open(&path.as_str())?;
        self.documents.insert(path, doc);
        Ok(())
    }

    /// Updates all [`Document`] items within the [`Library`].
    //
    /// [`Document`]: Document
    /// [`Library`]: Library
    #[must_use]
    pub fn update(self) -> Result<Self> {
        Ok(Self {
            documents: self
                .documents
                .into_iter()
                .map(|(p, d)| -> Result<(String, Document)> {
                    let doc = d.update(&p)?;
                    Ok((p, doc))
                })
                .filter_map(result::Result::ok)
                .collect(),
        })
    }
}

/// Holds infomation about a markdown document.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    name: String,
    hash: u64,
    mod_time: time::OffsetDateTime,
}

impl Document {
    /// Opens the given path and reads it for info, this will set the
    /// modification time to the current time and as such should be avoided in
    /// favor of using methods of [`Library`].
    ///
    /// [`Library`]: Library
    #[must_use]
    pub fn open(path: &impl AsRef<Path>) -> Result<Self> {
        let content = MdContent::new(fs::read_to_string(path).map_err(|_| Error::FileReadError)?);
        Ok(Self {
            name: content.title().unwrap_or("".to_owned()),
            hash: content.fnv1_hash(),
            mod_time: time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()),
        })
    }

    /// Updates the given [`Document`] by comparing its stored hash with that of
    /// the given [`MdContent`], if they are unequal then the modification time
    /// is updated to be the current time and the stored hash is updated.
    ///
    /// [`Document`]: Document
    /// [`MdContent`]: MdContent
    #[must_use]
    pub fn update(self, path: &impl AsRef<Path>) -> Result<Self> {
        let content = MdContent::new(fs::read_to_string(path).map_err(|_| Error::FileReadError)?);
        let new_hash = content.fnv1_hash();

        Ok(match self.hash == new_hash {
            true => self,
            false => Self {
                name: content.title().unwrap_or("".to_owned()),
                hash: new_hash,
                mod_time: time::OffsetDateTime::now_local()
                    .unwrap_or(time::OffsetDateTime::now_utc()),
                ..self
            },
        })
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
    pub fn name(&self) -> Cow<String> {
        Cow::Borrowed(&self.name)
    }
}

/// Represents a result of some library related function.
pub type Result<T> = result::Result<T, Error>;

/// Represents a library error.
#[derive(Debug, Clone, Copy)]
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
    /// [`OsString`]: fs::OsString
    InvalidStringError,

    /// Could not deserialize a struct from given TOML.
    InvalidTomlError,

    /// I/O failure to read a directory.
    DirectoryReadError,

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
        Self::InvalidStringError
    }
}

impl From<toml::ser::Error> for Error {
    fn from(_: toml::ser::Error) -> Self {
        Self::SerializationError
    }
}

impl From<toml::de::Error> for Error {
    fn from(_: toml::de::Error) -> Self {
        Self::InvalidTomlError
    }
}
