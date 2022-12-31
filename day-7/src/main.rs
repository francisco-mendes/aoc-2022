use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1,
        line_ending,
        multispace0,
        not_line_ending,
        space0,
        space1,
        u64,
    },
    multi::many1,
    Parser,
};

#[derive(Debug)]
enum Cd<'s> {
    Root,
    Parent,
    Dir(&'s str),
}

#[derive(Debug)]
enum Entry<'s> {
    Cd(Cd<'s>),
    Ls,
    Dir(&'s str),
    File(&'s str, usize),
}

fn parse_cd(input: &str) -> nom::IResult<&str, Cd<'_>> {
    let (input, _) = tag("$")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("cd")(input)?;
    let (input, _) = space1(input)?;

    let (input, cd) = alt((
        tag("/").map(|_| Cd::Root),
        tag("..").map(|_| Cd::Parent),
        alpha1.map(Cd::Dir),
    ))(input)?;

    Ok((input, cd))
}

fn parse_ls(input: &str) -> nom::IResult<&str, ()> {
    let (input, _) = tag("$")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("ls")(input)?;
    let (input, _) = space0(input)?;

    Ok((input, ()))
}

fn parse_dir(input: &str) -> nom::IResult<&str, &str> {
    let (input, _) = tag("dir")(input)?;
    let (input, _) = space1(input)?;
    let (input, dir) = not_line_ending(input)?;

    Ok((input, dir))
}

fn parse_file(input: &str) -> nom::IResult<&str, (&str, usize)> {
    let (input, size) = u64(input)?;
    let (input, _) = space1(input)?;
    let (input, name) = not_line_ending(input)?;

    Ok((input, (name, size as usize)))
}

fn parse_line(input: &str) -> nom::IResult<&str, Entry<'_>> {
    let (input, entry) = alt((
        parse_cd.map(Entry::Cd),
        parse_ls.map(|_| Entry::Ls),
        parse_dir.map(Entry::Dir),
        parse_file.map(|(name, size)| Entry::File(name, size)),
    ))(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, entry))
}

fn parse_input(input: &str) -> nom::IResult<&str, Vec<Entry<'_>>> {
    let (input, _) = multispace0(input)?;
    let (input, entries) = many1(parse_line)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, entries))
}

fn main() {
    const INPUT: &str = include_str!("input/given.txt");
    const MAX_DIR_SIZE: usize = 100_000;
    const MAX_SPACE_USE: usize = 40_000_000;

    let mut path = PathBuf::new();
    let mut dir_sizes: HashMap<_, usize> = HashMap::new();

    for entry in parse_input(INPUT).unwrap().1 {
        match entry {
            Entry::Cd(cd) => match cd {
                Cd::Root => path.push("/"),
                Cd::Parent => {
                    path.pop();
                }
                Cd::Dir(dir) => path.push(dir),
            },
            Entry::File(_, size) => {
                path.ancestors()
                    .for_each(|dir| *dir_sizes.entry(dir.to_owned()).or_default() += size);
            }
            _ => {}
        }
    }

    let total: usize = dir_sizes
        .values()
        .copied()
        .filter(|&size| size < MAX_DIR_SIZE)
        .sum();
    println!("The sum of the total size is {total}");

    let space_needed = dir_sizes[Path::new("/")].saturating_sub(MAX_SPACE_USE);

    let size = dir_sizes
        .values()
        .copied()
        .filter(|&size| size > space_needed)
        .min()
        .unwrap();
    println!("The size of the minimum candidate is {size}");
}
