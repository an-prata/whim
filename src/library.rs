// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::{fnv1_hash::Hashable, md_content::MdContent};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::Path};
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
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Reads a serialized [`Library`] from a file with the given path.
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(toml::from_str(fs::read_to_string(path)?.as_str())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Could not parse TOML"))?)
    }

    /// Saves the [`Library`], in TOML format, to the given file path.
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        fs::write(
            path,
            toml::to_string(self).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Could not serialize TOML")
            })?,
        )
    }

    /// Opens a [`Document`] at the given path and adds it to the [`Library`].
    ///
    /// [`Document`]: Document
    /// [`Library`]: Library
    pub fn add_doc(&mut self, path: String) -> io::Result<()> {
        let doc = Document::open(path.as_str())?;
        self.documents.insert(path, doc);
        Ok(())
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
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = MdContent::new(fs::read_to_string(path)?);
        Ok(Self {
            name: content.title().unwrap_or("".to_owned()),
            hash: content.hash(),
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
    pub fn update(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let content = MdContent::new(fs::read_to_string(path)?);
        let new_hash = content.hash();

        if self.hash != new_hash {
            self.name = content.title().unwrap_or("".to_owned());
            self.hash = new_hash;
            self.mod_time =
                time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc());
        }

        Ok(())
    }

    /// Gets the time of the last modification as made by either the struct's
    /// construction or an update.
    #[inline]
    #[must_use]
    pub fn mod_time(&self) -> time::OffsetDateTime {
        self.mod_time
    }
}
