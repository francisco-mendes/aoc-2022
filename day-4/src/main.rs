use std::ops::RangeInclusive;

pub mod parser {
    pub use nom::{
        self,
        character::complete::*,
        combinator::*,
        multi::*,
        sequence::*,
    };

    pub type Result<'i, T> = nom::IResult<&'i str, T>;
}

type Range = RangeInclusive<u32>;

fn parse_range(input: &str) -> parser::Result<'_, Range> {
    parser::map(
        parser::separated_pair(parser::u32, parser::char('-'), parser::u32),
        |(a, b)| a..=b,
    )(input)
}

fn parse_line(input: &str) -> parser::Result<'_, (Range, Range)> {
    parser::separated_pair(parse_range, parser::char(','), parse_range)(input)
}

fn parse_input(input: &str) -> parser::Result<'_, Vec<(Range, Range)>> {
    parser::delimited(
        parser::multispace0,
        parser::separated_list0(parser::line_ending, parse_line),
        parser::multispace0,
    )(input)
}

fn contains_whole((left, right): &(Range, Range)) -> bool {
    (left.contains(right.start()) && left.contains(right.end()))
        || (right.contains(left.start()) && right.contains(left.end()))
}

fn overlaps((left, right): &(Range, Range)) -> bool {
    left.contains(right.start())
        || left.contains(right.end())
        || right.contains(left.start())
        || right.contains(left.end())
}

fn count(
    (redundant_count, overlap_count): (usize, usize),
    schedules: (Range, Range),
) -> (usize, usize) {
    let redundant_incr = usize::from(contains_whole(&schedules));
    let overlap_incr = usize::from(overlaps(&schedules));

    (
        redundant_count + redundant_incr,
        overlap_count + overlap_incr,
    )
}

fn main() {
    const INPUT: &str = include_str!("input/given.txt");

    let (redundant, overlapping) = parse_input(INPUT)
        .unwrap()
        .1
        .into_iter()
        .fold((0, 0), count);

    println!("the number of fully contained assignments is {redundant}");
    println!("the number of overlapping assignments is {overlapping}");
}

#[cfg(test)]
mod test {
    use nom::{
        error::{
            Error,
            ErrorKind,
        },
        Err,
    };

    use super::*;

    #[test]
    fn parse_range_fails() {
        const INPUT: &str = "9-";

        let result = parse_range(INPUT);

        assert_eq!(
            result,
            Err(Err::Error(Error { input: "", code: ErrorKind::Digit }))
        );
    }

    #[test]
    fn parse_range_works() {
        const INPUT: &str = "9-12";

        let result = parse_range(INPUT);

        assert_eq!(result, Ok(("", 9..=12)));
    }

    #[test]
    fn parse_line_works() {
        const INPUT: &str = "9-12,11-13";

        let result = parse_line(INPUT);

        assert_eq!(result, Ok(("", (9..=12, 11..=13))));
    }

    #[test]
    fn parse_input_works() {
        const INPUT: &str = "9-12,11-13\n9-12,11-13";

        let result = parse_input(INPUT);

        assert_eq!(result, Ok(("", vec![(9..=12, 11..=13), (9..=12, 11..=13)])));
    }

    #[test]
    fn example_works() {
        const INPUT: &str = include_str!("input/example.txt");

        let (rest, ranges) = parse_input(INPUT).unwrap();
        let (redundant, overlapping) = ranges.into_iter().fold((0, 0), count);

        assert_eq!(rest, "");
        assert_eq!(redundant, 2);
        assert_eq!(overlapping, 4);
    }

    #[test]
    fn given_works() {
        const INPUT: &str = include_str!("input/given.txt");

        let (rest, ranges) = parse_input(INPUT).unwrap();
        let (redundant, overlapping) = ranges.into_iter().fold((0, 0), count);

        assert_eq!(rest, "");
        assert_eq!(redundant, 532);
        assert_eq!(overlapping, 854);
    }
}
