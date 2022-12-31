use core::slice::GetManyMutError;
use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
    },
    iter,
    sync::OnceLock,
};

use regex::Regex;
use snafu::prelude::*;

use crate::{
    command::Command,
    crane,
    crane::Crane,
};

#[non_exhaustive]
#[derive(Debug, Snafu)]
#[snafu(module(error), context(suffix(false)))]
pub enum ExecuteError {
    #[snafu(display("cannot move to the same stack {stack}"))]
    SameStack { stack: usize },
    #[snafu(display("cannot index stacks {} and {} from {len} total", indices[0], indices[1]))]
    OutOfBounds {
        source: GetManyMutError<2>,
        len: usize,
        indices: [usize; 2],
    },
    #[snafu(display("stack {from} does not have enough items"))]
    MissingItems {
        source: crane::MissingItemsError,
        from: usize,
    },
}

pub type Stack = Vec<u8>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stacks(Vec<Stack>);

impl Stacks {
    pub fn from_input(input: &str) -> Self {
        let mut stacks = Self(Vec::new());
        input
            .lines()
            .rev()
            .map(parse_stack_line)
            .for_each(|line| stacks.push_line(line));
        stacks
    }

    fn push_line(&mut self, line: Vec<Option<u8>>) {
        if self.0.len() < line.len() {
            self.0
                .extend(iter::repeat(Vec::new()).take(line.len() - self.0.len()))
        }

        for (stack, item) in iter::zip(&mut self.0, line) {
            let Some(item) = item else { continue };
            stack.push(item);
        }
    }
}

fn parse_stack_line(line: &str) -> Vec<Option<u8>> {
    static STACK_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = STACK_REGEX
        .get_or_init(|| unsafe { Regex::new(r#"\[([A-Z])]\s?|\s{3,4}"#).unwrap_unchecked() });

    regex
        .captures_iter(line)
        .map(|item| {
            item.get(1).and_then(|item| {
                item.as_str()
                    .as_bytes()
                    .first()
                    .copied()
                    .filter(u8::is_ascii_uppercase)
            })
        })
        .collect()
}

impl Display for Stacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "[")?;
        for stack in &self.0 {
            writeln!(f, "    {stack:?}")?;
        }
        writeln!(f, "]")
    }
}

impl From<&[&[u8]]> for Stacks {
    fn from(value: &[&[u8]]) -> Self {
        Stacks(value.iter().map(|stack| stack.to_vec()).collect())
    }
}

impl Stacks {
    pub fn execute<C: Crane>(
        &mut self,
        Command { n, from, to }: Command,
    ) -> Result<(), ExecuteError> {
        ensure!(from != to, error::SameStack { stack: from });

        let len = self.0.len();
        let [origin, dest] = self
            .0
            .get_many_mut([from, to])
            .context(error::OutOfBounds { len, indices: [from, to] })?;

        C::move_items(n, origin, dest).context(error::MissingItems { from })
    }

    pub fn items_on_top(&self) -> String {
        self.0
            .iter()
            .filter_map(|stack| stack.last().copied())
            .map(char::from)
            .collect()
    }
}

#[cfg(test)]
mod test {
    use yare::parameterized;

    use super::*;
    use crate::crane::BaseCrane;

    #[parameterized(
        works       = { "[Z] [M] [P]", vec![Some(b'Z'), Some(b'M'), Some(b'P')] },
        empty       = { "",            vec![] },
        hole        = { "    [D]",     vec![None, Some(b'D')] },
        small_hole  = { "  [D]",       vec![Some(b'D')] },
        invalid     = { "uwu",         vec![] },
    )]
    fn parse_stack_line(input: &str, expected: Vec<Option<u8>>) {
        let value = parse_stack_line(input);
        assert_eq!(value, expected);
    }

    #[parameterized(
        works = {
            &[],
            vec![Some(b'Z'), Some(b'M'), Some(b'P')],
            &[b"Z", b"M", b"P"],
        },
        empty_line      = { &[], vec![],                 &[] },
        line_with_hole  = { &[], vec![None, Some(b'D')], &[b"", b"D"] },
        line_with_nones = { &[], vec![None, None],       &[b"", b""] },
        update_stacks = {
            &[b"Z", b"M", b"P"],
            vec![Some(b'Z'), Some(b'M'), Some(b'P')],
            &[b"ZZ", b"MM", b"PP"],
        },
        update_with_hole = {
            &[b"Z", b"M", b"P"],
            vec![Some(b'Z'), None, Some(b'P')],
            &[b"ZZ", b"M", b"PP"],
        },
        update_with_partial_line = {
            &[b"Z", b"M", b"P"],
            vec![Some(b'Z')],
            &[b"ZZ", b"M", b"P"],
        },
        update_expand_stacks = {
            &[b"Z", b"M", b"P"],
            vec![None, None, None, Some(b'Y')],
            &[b"Z", b"M", b"P", b"Y"],
        },
    )]
    fn push_line(stacks: &[&[u8]], line: Vec<Option<u8>>, expected: &[&[u8]]) {
        let mut stacks = Stacks::from(stacks);

        stacks.push_line(line);

        assert_eq!(stacks, Stacks::from(expected));
    }

    #[parameterized(
        works     = { [b"Z", b"M",   b"P"], [1, 1, 0], [b"ZM",   b"",    b"P"] },
        move_many = { [b"Z", b"MXC", b"P"], [2, 1, 0], [b"ZCX",  b"M",   b"P"] },
        move_all  = { [b"Z", b"MXC", b"P"], [3, 1, 0], [b"ZCXM", b"",    b"P"] },
        move_none = { [b"Z", b"MXC", b"P"], [0, 1, 0], [b"Z",    b"MXC", b"P"] },
    )]
    fn execute(stacks: [&[u8]; 3], cmd: [usize; 3], expected: [&[u8]; 3]) {
        let cmd = Command::from(cmd);
        let mut stacks = Stacks::from(stacks.as_slice());

        stacks.execute::<BaseCrane>(cmd).unwrap();

        assert_eq!(stacks, Stacks::from(expected.as_slice()));
    }

    #[test]
    fn execute_not_enough() {
        let stacks: &[&[_]] = &[b"Z", b"M", b"P"];
        let mut stacks = Stacks::from(stacks);

        let error = stacks
            .execute::<BaseCrane>(Command::from([2, 2, 1]))
            .unwrap_err();

        if let ExecuteError::MissingItems { from, .. } = error {
            assert_eq!(from, 2);
        } else {
            panic!("expected missing items");
        }
    }

    #[test]
    fn execute_same_stack() {
        let stacks: &[&[_]] = &[b"Z", b"M", b"P"];
        let mut stacks = Stacks::from(stacks);
        let cmd = Command::from([1, 2, 2]);

        let error = stacks.execute::<BaseCrane>(cmd).unwrap_err();

        if let ExecuteError::SameStack { stack } = error {
            assert_eq!(stack, 2);
        } else {
            panic!("expected same stack")
        }
    }

    #[parameterized(
        from_oob   = { [b"Z", b"M", b"P"], [1, 3, 2] },
        to_oob     = { [b"Z", b"M", b"P"], [1, 2, 3] },
    )]
    fn execute_out_of_bounds(stacks: [&[u8]; 3], line: [usize; 3]) {
        let mut stacks = Stacks::from(stacks.as_slice());
        let cmd = Command::from(line);

        let error = stacks.execute::<BaseCrane>(cmd).unwrap_err();

        if let ExecuteError::OutOfBounds { len, indices, .. } = error {
            assert_eq!(len, 3);
            assert_eq!(indices, line[1..]);
        } else {
            panic!("expected out of bounds");
        }
    }
}
