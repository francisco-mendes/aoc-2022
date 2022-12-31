#![feature(once_cell, iter_collect_into)]
#![feature(get_many_mut)]
#![feature(array_try_from_fn)]
#![feature(assert_matches)]
#![feature(try_blocks)]

use std::str::FromStr;

use snafu::prelude::*;

use crate::{
    command::Command,
    crane::{
        BaseCrane,
        ManyCrane,
    },
    stacks::Stacks,
};

mod command;
mod crane;
mod stacks;

#[non_exhaustive]
#[derive(Debug, Snafu)]
#[snafu(module(error), context(suffix(false)))]
pub enum Error {
    #[snafu(display("unable to parse command"))]
    ParseCommand { source: command::Error },
    #[snafu(display("unable to execute command"))]
    ExecuteCommand { source: stacks::ExecuteError },
}

fn main() -> Result<(), Error> {
    const STACK: &str = include_str!("stacks/given.txt");
    const CMDS: &str = include_str!("commands/given.txt");

    let mut stacks = Stacks::from_input(STACK);
    let mut rev_stacks = stacks.clone();

    for cmd in CMDS.lines().map(Command::from_str) {
        let cmd = cmd.context(error::ParseCommand)?;
        rev_stacks
            .execute::<BaseCrane>(cmd)
            .context(error::ExecuteCommand)?;
        stacks
            .execute::<ManyCrane>(cmd)
            .context(error::ExecuteCommand)?;
    }

    let rev_top: String = rev_stacks.items_on_top();
    let top: String = stacks.items_on_top();

    println!("The top crates are {rev_top} if moving one by one.");
    println!("The top crates are {top} if moving many at a time.");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_works() {
        const STACK: &str = include_str!("stacks/example.txt");
        const CMDS: &str = include_str!("commands/example.txt");

        let mut stacks = Stacks::from_input(STACK);
        let mut rev_stacks = stacks.clone();

        for cmd in CMDS.lines().map(Command::from_str) {
            let cmd = cmd.unwrap();
            rev_stacks.execute::<BaseCrane>(cmd).unwrap();
            stacks.execute::<ManyCrane>(cmd).unwrap();
        }

        assert_eq!(rev_stacks.items_on_top(), "CMZ");
        assert_eq!(stacks.items_on_top(), "MCD");
    }
}
