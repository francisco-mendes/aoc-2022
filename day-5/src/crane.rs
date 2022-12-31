use snafu::prelude::*;

use crate::stacks::Stack;

#[non_exhaustive]
#[derive(Debug, Snafu)]
#[snafu(module(error), context(suffix(false)))]
#[snafu(display("cannot move {count} items from stack with {len} items"))]
pub struct MissingItemsError {
    count: usize,
    len: usize,
}

pub trait Crane {
    fn move_items(n: usize, from: &mut Stack, to: &mut Stack) -> Result<(), MissingItemsError>;
}

pub struct BaseCrane;

impl Crane for BaseCrane {
    fn move_items(n: usize, from: &mut Stack, to: &mut Stack) -> Result<(), MissingItemsError> {
        ensure!(
            from.len() >= n,
            error::MissingItems { count: n, len: from.len() }
        );

        from.drain((from.len() - n)..).rev().collect_into(to);
        Ok(())
    }
}

pub struct ManyCrane;

impl Crane for ManyCrane {
    fn move_items(n: usize, from: &mut Stack, to: &mut Stack) -> Result<(), MissingItemsError> {
        ensure!(
            from.len() >= n,
            error::MissingItems { count: n, len: from.len() }
        );

        from.drain((from.len() - n)..).collect_into(to);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use yare::parameterized;

    use super::*;

    #[parameterized(
        works       = { 1, [b"A".to_vec(),   b"B".to_vec()], [b"".to_vec(),  b"BA".to_vec()] },
        move_many   = { 2, [b"ABC".to_vec(), b"D".to_vec()], [b"A".to_vec(), b"DCB".to_vec()] },
        move_all    = { 3, [b"ABC".to_vec(), b"D".to_vec()], [b"".to_vec(),  b"DCBA".to_vec()] },
        move_none   = { 0, [b"A".to_vec(),   b"B".to_vec()], [b"A".to_vec(), b"B".to_vec()] },
    )]
    fn move_base_crane(n: usize, swaps: [Stack; 2], expected: [Stack; 2]) {
        let [mut from, mut to] = swaps;

        BaseCrane::move_items(n, &mut from, &mut to).unwrap();

        assert_eq!([from, to], expected);
    }

    #[parameterized(
        works       = { 1, [b"A".to_vec(),   b"B".to_vec()], [b"".to_vec(),  b"BA".to_vec()] },
        move_many   = { 2, [b"ABC".to_vec(), b"D".to_vec()], [b"A".to_vec(), b"DBC".to_vec()] },
        move_all    = { 3, [b"ABC".to_vec(), b"D".to_vec()], [b"".to_vec(),  b"DABC".to_vec()] },
        move_none   = { 0, [b"A".to_vec(),   b"B".to_vec()], [b"A".to_vec(), b"B".to_vec()] },
    )]
    fn move_many_crane(n: usize, swaps: [Stack; 2], expected: [Stack; 2]) {
        let [mut from, mut to] = swaps;

        ManyCrane::move_items(n, &mut from, &mut to).unwrap();

        assert_eq!([from, to], expected);
    }

    #[test]
    fn base_crane_move_too_many() {
        let [mut from, mut to] = [b"A".to_vec(), b"B".to_vec()];

        let actual = BaseCrane::move_items(2, &mut from, &mut to).unwrap_err();

        assert_eq!(actual.count, 2);
        assert_eq!(actual.len, 1);
    }

    #[test]
    fn many_crane_move_too_many() {
        let [mut from, mut to] = [b"A".to_vec(), b"B".to_vec()];

        let actual = ManyCrane::move_items(2, &mut from, &mut to).unwrap_err();

        assert_eq!(actual.count, 2);
        assert_eq!(actual.len, 1);
    }
}
