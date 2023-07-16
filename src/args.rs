// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

use std::{collections::HashMap, error, fmt, rc::Rc, result};

/// Parses command line arguments based on given commands and flags.
#[derive(Debug)]
pub struct ArgsParser<T, I>
where
    T: Iterator<Item = I>,
    I: AsRef<str>,
{
    args: T,
    commands: Vec<Command>,
    flags: Vec<Flag>,
}

impl<T, I> ArgsParser<T, I>
where
    T: Iterator<Item = I>,
    I: AsRef<str>,
{
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

        // Takes an argument and tries to parse it as a `Flag`.
        let try_parse_flag = |arg: &str| -> Result<ArgsItem> {
            let flag = match arg.starts_with("--") {
                true => arg.replace("--", ""),
                false if arg.len() == "-f".len() => arg.replace('-', ""),
                _ => return Err(Error::MalformedArgument(arg.into())),
            };

            match self.flags.iter().find(|f| f.name() == flag.as_str()) {
                Some(f) => Ok(ArgsItem::Flag(f.to_owned())),
                None => Err(Error::BadFlag),
            }
        };

        for arg in self.args {
            let arg = arg.as_ref();

            prev = match prev {
                ArgsItem::Flag(flag @ Flag::Bool(_)) => {
                    match self.commands.iter().find(|c| &*c.0 == arg) {
                        Some(c) => ArgsItem::Command(c.clone()),
                        None => match arg.starts_with('-') {
                            true => try_parse_flag(arg)?,
                            false => ArgsItem::Value(flag.parse_value(arg)?),
                        },
                    }
                }
                ArgsItem::Flag(flag) => ArgsItem::Value(flag.parse_value(arg)?),
                _ => match self.commands.iter().find(|c| &*c.0 == arg) {
                    Some(c) => ArgsItem::Command(c.clone()),
                    None => match arg.starts_with('-') {
                        true => try_parse_flag(arg)?,
                        false => ArgsItem::Value(Value::String(arg.to_owned())),
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
                    Some(ArgsItem::Value(v)) => map.insert(f.clone(), Some(v.clone())),
                    _ => match f {
                        Flag::Bool(_) => map.insert(f.clone(), Some(Value::Bool(true))),
                        _ => map.insert(f.clone(), None),
                    },
                },
                _ => continue,
            };
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

/// Represent a command line flag, [`Flag`]s with single character names may be
/// used with a single dash (e.g. '-f') or double dash (e.g. '--f'). A [`Flag`]
/// who's name exceeds a single character must be preceded by two dashes
/// (e.g. '--flag').
///
/// # Examples
///
/// ```
/// use args::*;
///
/// let args = vec!["program_name", "-f", "123"];
/// let flag = Flag::Int("f".into())
/// let parsed_args = ArgsParser::new(args)
///     .flag(flag.clone())
///     .parse()
///     .unwrap();
///
/// let parsed_flags = parsed_args.flags();
/// assert_eq!(parsed_flags[&flag], Some(Value::Int(123)));
/// ```
///
/// [`Flag`]: Flag
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    /// The [`Bool`] variant may take an argument, but its presence will imply
    /// a value of `true` otherwise, though its absence does not imply a value
    /// of `false`, instead [`ParsedArgs`] will map the [`Flag`] to [`None`].
    ///
    /// [`Bool`]: Flag::Bool
    /// [`ParsedArgs`]: ParsedArgs
    /// [`Flag`]: Flag
    /// [`None`]: None
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

    /// Returns true if the [`Flag`]'s name is a single character.
    ///
    /// [`Flag`]: Flag
    #[must_use]
    pub fn single_char(&self) -> bool {
        self.name().len() == 1
    }

    /// Parses an argument into a [`Value`] of a variant that coorasponds to the
    /// variant of this [`Flag`].
    ///
    /// [`Flag`]: Flag
    /// [`Value`]: Value
    #[must_use]
    pub fn parse_value(&self, arg: &str) -> Result<Value> {
        Ok(match self {
            Flag::Bool(_) => Value::Bool(
                arg.parse()
                    .map_err(|_| Error::MalformedArgument(arg.into()))?,
            ),
            Flag::Uint(_) => Value::Uint(
                arg.parse()
                    .map_err(|_| Error::MalformedArgument(arg.into()))?,
            ),
            Flag::Int(_) => Value::Int(
                arg.parse()
                    .map_err(|_| Error::MalformedArgument(arg.into()))?,
            ),
            Flag::String(_) => Value::String(
                arg.parse()
                    .map_err(|_| Error::MalformedArgument(arg.into()))?,
            ),
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
    /// string given to a [`Flag::Int`] flag. The argument determined to be
    /// malformed is included as the value of this [`MalformedArgument`].
    ///
    /// [`Flag::Int`]: Flag::Int
    /// [`MalformedArgument`]: Error::MalformedArgument
    MalformedArgument(Rc<str>),

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
            "program", "command", "--flag0", "123", "--flag1", "true", "-f", "--flag4", "command",
            "--flag5", "-2",
        ];

        let flag0 = Flag::Uint("flag0".into());
        let flag1 = Flag::Bool("flag1".into());
        let flag2 = Flag::Int("flag2".into());
        let flag3 = Flag::Bool("f".into());
        let flag4 = Flag::String("flag4".into());
        let flag5 = Flag::Int("flag5".into());
        let cmd = Command("command".into());

        let parsed_args = ArgsParser::new(args.into_iter())
            .flag(flag0.clone())
            .flag(flag1.clone())
            .flag(flag2.clone())
            .flag(flag3.clone())
            .flag(flag4.clone())
            .flag(flag5.clone())
            .command(cmd.clone())
            .parse()
            .unwrap();

        let flags = parsed_args.flags();

        assert_eq!(flags[&flag0], Some(Value::Uint(123)));
        assert_eq!(flags[&flag1], Some(Value::Bool(true)));
        assert_eq!(flags[&flag2], None);
        assert_eq!(flags[&flag3], Some(Value::Bool(true)));
        assert_eq!(flags[&flag4], Some(Value::String("command".to_owned())));
        assert_eq!(flags[&flag5], Some(Value::Int(-2)));

        let commands = parsed_args.commands();

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], cmd);
    }
}
