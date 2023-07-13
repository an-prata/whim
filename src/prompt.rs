// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use std::{
    fmt::{self, Display},
    io, result,
};

/// A yes or no prompt defaulting to yes.
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum Yes {
    /// The user did not give a "no" response.
    #[default]
    Yes,

    /// The user gave a "no" response.
    No,
}

impl PromptItem for Yes {
    const OPTIONS: &'static str = "Y/n";

    fn parse_input(input: String) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "n" | "no" => Ok(Self::No),
            _ => Ok(Self::Yes),
        }
    }
}

/// A yes or no prompt defaulting to yes.
#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub enum No {
    /// The user did not give a "yes" response.
    #[default]
    No,

    /// The user gave a "yes" response.
    Yes,
}

impl PromptItem for No {
    const OPTIONS: &'static str = "y/N";

    fn parse_input(input: String) -> Result<Self> {
        match input.to_lowercase().as_str() {
            "y" | "yes" => Ok(Self::Yes),
            _ => Ok(Self::No),
        }
    }
}

/// Represents a item that can be constructed based off of prompted user input.
pub trait PromptItem: Sized {
    /// Options string to present to the user. A yes/no prompt could use these:
    /// "Y/n", "y/N", "Yes/no", "YES/no", etc. Capatalize an option if it is a
    /// default.
    ///
    /// It would also be reasonable to explain the type of data, for instance if
    /// constructing an [`i32`]: "integer" or "int" may be appropriate.
    ///
    /// [`i32`]: i32
    const OPTIONS: &'static str;

    /// Outputs a prompt to the user and waits for input, then creates a new
    /// [`Self`].
    ///
    /// # Errors
    ///
    /// This function may return an error if one is encountered when reading
    /// from [`std::io::stdin`], or if parsing fails and cannot provide a
    /// default.
    ///
    /// [`Self`]: Self
    /// [`std::io::stdin`]: io::stdin
    fn from_prompt(prompt: String, suffix: Option<char>) -> Result<Self> {
        match suffix {
            Some(c) => print!("{} [{}] {} ", prompt, Self::OPTIONS, c),
            None => print!("{} [{}] ", prompt, Self::OPTIONS),
        }

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|_| Error)?;

        Ok(Self::parse_input(input)?)
    }

    /// Given an input [`String`], returns a [`PromptItem`]. Should return a
    /// reasonable default if possible, e.g. the prompt "[Y/n] ? ", given the
    /// input 'a' could reasonably give a "Yes" since it would be the default by
    /// convention. This applies to blank input as well.
    ///
    /// If an option is a default it should be capatalized.
    ///
    /// [`String`]: String
    /// [`PromptItem`]: PromptItem
    fn parse_input(input: String) -> Result<Self>;
}

/// [`Result`] type alias for [`PromptItem`] structs.
///
/// [`Result`]: result::Result
/// [`PromptItem`]: PromptItem
pub type Result<T> = result::Result<T, Error>;

/// An error for prompts.
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error getting input or parsing it.")
    }
}
