#![feature(once_cell)]

use regex::Regex;
use std::ops::RangeInclusive;
use std::sync::OnceLock;

fn parse_line(line: &str) -> (RangeInclusive<usize>, RangeInclusive<usize>) {
    static LINE_REGEX: OnceLock<Regex> = OnceLock::new();

    let regex = LINE_REGEX.get_or_init(|| Regex::new(r#"(\d+)-(\d+),(\d+)-(\d+)"#).unwrap());

    let captures = regex.captures(line).unwrap();

    (
        captures[1].parse().unwrap()..=captures[2].parse().unwrap(),
        captures[3].parse().unwrap()..=captures[4].parse().unwrap(),
    )
}

fn contains_whole((left, right): &(RangeInclusive<usize>, RangeInclusive<usize>)) -> bool {
    (left.contains(right.start()) && left.contains(right.end()))
        || (right.contains(left.start()) && right.contains(left.end()))
}

fn overlaps((left, right): &(RangeInclusive<usize>, RangeInclusive<usize>)) -> bool {
    left.contains(right.start())
        || left.contains(right.end())
        || right.contains(left.start())
        || right.contains(left.end())
}

fn count((redundant_count, overlap_count): (usize, usize), schedules: (RangeInclusive<usize>, RangeInclusive<usize>)) -> (usize, usize) {
    let redundant_incr = usize::from(contains_whole(&schedules));
    let overlap_incr = usize::from(overlaps(&schedules));

    (redundant_count + redundant_incr, overlap_count + overlap_incr)
}

fn main() {
    const INPUT: &str = include_str!("input/given.txt");

    let (redundant, overlapping) = INPUT
        .lines()
        .map(parse_line)
        .fold((0, 0), count);

    println!("the number of fully contained assignments is {redundant}");
    println!("the number of overlapping assignments is {overlapping}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_works() {
        const INPUT: &str = include_str!("input/given.txt");

        let (redundant, overlapping) = INPUT
            .lines()
            .map(parse_line)
            .fold((0, 0), count);

        assert_eq!(redundant, 532);
        assert_eq!(overlapping, 854);
    }
}