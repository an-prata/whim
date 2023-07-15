// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

pub trait Href {
    /// Returns the number of items in the given [`Href`].
    ///
    /// "/index.html" -> 1
    /// "/blog/" -> 2
    /// "../x" -> 2
    ///
    /// [`Href`]: Href
    fn path_items(&self) -> usize;

    /// Returns the number of parent accessors, or ".." path items, in the given
    /// [`Href`].
    ///
    /// [`Href`]: Href
    fn parent_accessors(&self) -> usize;
}

impl Href for String {
    fn path_items(&self) -> usize {
        match self.chars().next().unwrap_or_default() == '/' {
            true => self.matches('/').count(),
            false => self.matches('/').count() + 1,
        }
    }

    fn parent_accessors(&self) -> usize {
        self.matches("..").count()
    }
}
