use std::{
    array,
    num::{
        NonZeroUsize,
        ParseIntError,
    },
    str::FromStr,
    sync::OnceLock,
};

use regex::Regex;
use snafu::prelude::*;

#[non_exhaustive]
#[derive(Debug, Snafu)]
#[snafu(module(error), context(suffix(false)))]
pub enum Error {
    #[snafu(display("invalid command '{input}'"))]
    InvalidCommand { input: String },
    #[snafu(display("invalid {name} parameter '{arg}'"))]
    InvalidArg {
        source: ParseIntError,
        name: &'static str,
        arg: String,
    },
    #[snafu(display("cannot move from the stack {stack} to itself"))]
    SameSourceDest { stack: usize },
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Command {
    pub n: usize,
    pub from: usize,
    pub to: usize,
}

impl From<[usize; 3]> for Command {
    fn from([n, from, to]: [usize; 3]) -> Self {
        Self { n, from, to }
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        const NAMES: [&str; 3] = ["move", "from", "to"];

        static CMD_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = CMD_REGEX.get_or_init(|| unsafe {
            Regex::new(r#"move (?P<move>-?\d+) from (?P<from>-?\d+) to (?P<to>-?\d+)"#)
                .unwrap_unchecked()
        });

        let captures = regex
            .captures(line)
            .with_context(|| error::InvalidCommand { input: line })?;

        let [n, from, to]: [NonZeroUsize; 3] = array::try_from_fn(|i| {
            let name = NAMES[i];
            let arg = &captures[name];
            arg.parse().context(error::InvalidArg { name, arg })
        })?;

        ensure!(from != to, error::SameSourceDest { stack: from });

        Ok(Self {
            n: n.get(),
            from: from.get() - 1,
            to: to.get() - 1,
        })
    }
}

#[cfg(test)]
mod test {
    use std::num::IntErrorKind;

    use yare::parameterized;

    use super::*;

    #[parameterized(
        works       = { "move 1 from 2 to 1",  [1, 1, 0] },
        two_digits  = { "move 10 from 2 to 1", [10, 1, 0] },
        from_lower  = { "move 10 from 1 to 2", [10, 0, 1] },
    )]
    fn parse_command(input: &str, expected: [usize; 3]) {
        let cmd = Command::from_str(input).unwrap();

        assert_eq!(cmd, Command::from(expected));
    }

    #[parameterized(
        invalid         = { "uwu" },
        missing_move    = { "move from 2 to 1" },
        missing_from    = { "move 1 from to 1" },
        missing_to      = { "move 1 from 2 to" },
        missing_spaces  = { "move1from2to1" },
        just_minus      = { "move - from - to -" },
    )]
    fn parse_command_fails_input(input: &str) {
        let err = Command::from_str(input).unwrap_err();

        if let Error::InvalidCommand { input: actual } = err {
            assert_eq!(actual, input);
        } else {
            panic!("expected an invalid command")
        }
    }

    #[parameterized(
        negative_move   = { "move -1 from 2 to 1", "move", "-1", IntErrorKind::InvalidDigit },
        negative_from   = { "move 1 from -2 to 1", "from", "-2", IntErrorKind::InvalidDigit },
        negative_to     = { "move 1 from 2 to -1", "to",   "-1", IntErrorKind::InvalidDigit },
        zero_move       = { "move 0 from 2 to 1",  "move", "0",  IntErrorKind::Zero },
        zero_from       = { "move 1 from 0 to 1",  "from", "0",  IntErrorKind::Zero },
        zero_to         = { "move 1 from 2 to 0",  "to",   "0",  IntErrorKind::Zero },
    )]
    fn parse_command_fails_arg(
        input: &str,
        expected_name: &'static str,
        expected_arg: &str,
        expected_kind: IntErrorKind,
    ) {
        let err = Command::from_str(input).unwrap_err();

        if let Error::InvalidArg { arg, name, source } = err {
            assert_eq!(name, expected_name);
            assert_eq!(arg, expected_arg);
            assert_eq!(source.kind(), &expected_kind)
        } else {
            panic!("expected an invalid argument")
        }
    }
}
