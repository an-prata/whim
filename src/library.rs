// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use crate::{fnv1_hash::Hashable, md_content::MdContent};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::Path,
};
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
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(toml::from_str(fs::read_to_string(path)?.as_str())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Could not parse TOML"))?)
    }

    /// Saves the [`Library`], in TOML format, to the given file path.
    ///
    /// [`Library`]: Library
    #[inline]
    #[must_use]
    pub fn save<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        fs::write(
            path,
            toml::to_string(self).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Could not serialize TOML")
            })?,
        )
    }
}

/// Holds infomation about a markdown document.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Document {
    hash: u64,
    mod_time: time::OffsetDateTime,
}

impl Document {
    /// Creates a new [`Document`] given an instance of [`MdContent`]. The hash
    /// of the [`MdContent`] is stored and the modification time is set to the
    /// current time.
    ///
    /// [`Document`]: Document
    /// [`MdContent`]: MdContent
    #[inline]
    pub fn new(content: &MdContent) -> Self {
        Self {
            hash: content.hash(),
            mod_time: time::OffsetDateTime::now_local()
                .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
        }
    }

    /// Updates the given [`Document`] by comparing its stored hash with that of
    /// the given [`MdContent`], if they are unequal then the modification time
    /// is updated to be the current time and the stored hash is updated.
    ///
    /// [`Document`]: Document
    /// [`MdContent`]: MdContent
    #[inline]
    #[must_use]
    pub fn update(self, content: MdContent) -> Self {
        let new_hash = content.hash();
        match self.hash == new_hash {
            true => self,
            false => Self {
                hash: new_hash,
                mod_time: time::OffsetDateTime::now_local()
                    .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
            },
        }
    }

    /// Gets the time of the last modification as made by either the struct's
    /// construction or an update.
    #[inline]
    #[must_use]
    pub fn mod_time(&self) -> time::OffsetDateTime {
        self.mod_time
    }
}
