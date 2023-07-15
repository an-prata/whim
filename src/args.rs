// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use std::{collections::HashMap, error, fmt, rc::Rc, result};

/// Parses command line arguments based on given commands and flags.
#[derive(Debug)]
pub struct ArgsParser<T: Iterator<Item = String>> {
    args: T,
    commands: Vec<Command>,
    flags: Vec<Flag>,
}

impl<T: Iterator<Item = String>> ArgsParser<T> {
    /// Creates a new [`ArgsParser`] with the given [`Iterator`] over all
    /// arguments as [`String`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use args::ArgsParser;
    ///
    /// let parser = ArgsParser::new(std::env::args().unwrap());
    /// ```
    ///
    /// [`ArgsParser`]: ArgsParser
    /// [`Iterator`]: Iterator
    /// [`String`]: String
    #[must_use]
    pub fn new(args: T) -> Self {
        Self {
            args,
            commands: Vec::new(),
            flags: Vec::new(),
        }
    }

    /// Adds a [`Flag`] for parsing.
    ///
    /// [`Flag`]: Flag
    #[must_use]
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.push(flag);
        self
    }

    /// Adds a [`Command`] for parsing.
    ///
    /// [`Command`]: Command
    #[must_use]
    pub fn command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// Parses all previously given arguments for [`Flag`], [`Command`], and
    /// [`Value`] items corresponding to previously given [`Flag`] values and
    /// [`Command`] values. Returns a [`ParsedArgs`] struct.
    ///
    /// [`Flag`]: Flag
    /// [`Command`]: Command
    /// [`Value`]: Value
    /// [`ParsedArgs`]: ParsedArgs
    pub fn parse(self) -> Result<ParsedArgs> {
        let mut prev = ArgsItem::Value(Value::Bool(false));
        let mut items = Vec::new();

        for arg in self.args {
            prev = match arg.starts_with("--") {
                true => {
                    let flag = arg.replace("--", "");
                    match self.flags.iter().find(|f| f.name() == flag.as_str()) {
                        Some(f) => ArgsItem::Flag(f.to_owned()),
                        None => return Err(Error::BadFlag),
                    }
                }
                false => match self.commands.iter().find(|c| &*c.0 == arg.as_str()) {
                    Some(c) => ArgsItem::Command(c.to_owned()),
                    None => match prev {
                        ArgsItem::Flag(f) => ArgsItem::Value(f.parse_value(arg.as_str())?),
                        _ => ArgsItem::Value(Value::String(arg)),
                    },
                },
            };

            items.push(prev.clone());
        }

        Ok(ParsedArgs {
            flags: self.flags,
            items,
        })
    }
}

/// Holds arguments parsed by an [`ArgsParser`] and is made for the easy checking
/// of [`Value`]s attributed to [`Flag`]s and the state of [`Command`]s.
///
/// [`ArgsParser`]: ArgsParser
/// [`Flag`]: Flag
/// [`Value`]: Value
/// [`Command`]: Command
pub struct ParsedArgs {
    flags: Vec<Flag>,
    items: Vec<ArgsItem>,
}

impl ParsedArgs {
    /// Creates a [`HashMap`] with keys of type [`Flag`] and values of type
    /// [`Option`] which will contain either a [`Value`] or [`None`] depending
    /// on if a value was provided.
    ///
    /// [`HashMap`]: HashMap
    /// [`Flag`]: Flag
    /// [`Value`]: Value
    /// [`Option`]: Option
    /// [`None`]: None
    #[must_use]
    pub fn flags(&self) -> HashMap<Flag, Option<Value>> {
        let mut items = self.items.iter().peekable();
        let mut map = self
            .flags
            .iter()
            .map(|f| (f.clone(), None))
            .collect::<HashMap<_, _>>();

        while let Some(item) = items.next() {
            match item {
                ArgsItem::Flag(f) => match items.peek() {
                    Some(ArgsItem::Value(v)) => {
                        map.insert(f.clone(), Some(v.clone()));
                    }
                    _ => {
                        map.insert(f.clone(), None);
                    }
                },
                _ => continue,
            }
        }

        map
    }

    /// Gets a list of all [`Command`]s present in the parsed command line
    /// arguments.
    ///
    /// [`Command`]: Command
    #[must_use]
    pub fn commands(&self) -> Vec<Command> {
        self.items
            .iter()
            .filter_map(|i| match i {
                ArgsItem::Command(c) => Some(c.clone()),
                _ => None,
            })
            .collect()
    }
}

/// A single item of the given command line arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgsItem {
    /// Represents a command of the program, this is an argument that matches
    /// some given [`String`] but is not preceded by dashes.
    ///
    /// [`String`]: String
    Command(Command),

    /// Represents an argument that matches some given [`String`], and is
    /// preceded by two dashes. If the [`String`] is a single character long
    /// then it must be preceded by a single dash. [`Flag`] arguments may
    /// optionally be followed by a value.
    Flag(Flag),

    /// Any argument that either follows a [`Flag`] that expects a value, or
    /// does not match any given [`String`] for a [`Command`].
    Value(Value),
}

/// A subcommand of a program as given in command line arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command(pub Rc<str>);

/// Represents a command line flag, preceded by two dashes ("--"). Its value for
/// each variant is the flag's name.
///
/// "myapp --arg 123" -> `Flag::Uint("arg")`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    Bool(Rc<str>),
    Uint(Rc<str>),
    Int(Rc<str>),
    String(Rc<str>),
}

impl Flag {
    /// Returns this [`Flag`]'s name, regardless of variation.
    ///
    /// [`Flag`]: Flag
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Flag::Bool(s) => s,
            Flag::Uint(s) => s,
            Flag::Int(s) => s,
            Flag::String(s) => s,
        }
    }

    /// Parses an argument into a [`Value`] of a variant that coorasponds to the
    /// variant of this [`Flag`].
    ///
    /// [`Flag`]: Flag
    /// [`Value`]: Value
    #[must_use]
    pub fn parse_value(&self, arg: &str) -> Result<Value> {
        Ok(match self {
            Flag::Bool(_) => Value::Bool(arg.parse().map_err(|_| Error::MalformedArgument)?),
            Flag::Uint(_) => Value::Uint(arg.parse().map_err(|_| Error::MalformedArgument)?),
            Flag::Int(_) => Value::Int(arg.parse().map_err(|_| Error::MalformedArgument)?),
            Flag::String(_) => Value::String(arg.parse().map_err(|_| Error::MalformedArgument)?),
        })
    }
}

/// May hold any argument given as command line args.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Uint(u64),
    Int(i64),
    String(String),
}

/// The result type of argument parsing related functions.
type Result<T> = result::Result<T, Error>;

/// An error that may occure during the parsing of arguments.
#[derive(Debug)]
pub enum Error {
    /// At least one argument was incorrect for its position. e.g. an text
    /// string given to a [`Flag::Int`] flag.
    ///
    /// [`Flag::Int`]: Flag::Int
    MalformedArgument,

    /// An argument syntactically matches a what would be expected for a
    /// [`Flag`], but did not match any given [`Flag`] names.
    ///
    /// [`Flag`]: Flag
    BadFlag,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_test() {
        let args = vec![
            "program".to_owned(),
            "command".to_owned(),
            "--flag0".to_owned(),
            "123".to_owned(),
            "--flag1".to_owned(),
            "true".to_owned(),
        ];

        let flag0 = Flag::Uint("flag0".into());
        let flag1 = Flag::Bool("flag1".into());
        let flag2 = Flag::Int("flag2".into());
        let cmd = Command("command".into());

        let parsed_args = ArgsParser::new(args.into_iter())
            .flag(flag0.clone())
            .flag(flag1.clone())
            .flag(flag2.clone())
            .command(cmd.clone())
            .parse()
            .unwrap();

        let flags = parsed_args.flags();

        assert_eq!(flags[&flag0], Some(Value::Uint(123)));
        assert_eq!(flags[&flag1], Some(Value::Bool(true)));
        assert_eq!(flags[&flag2], None);

        let commands = parsed_args.commands();

        assert_eq!(commands[0], cmd);
    }
}
