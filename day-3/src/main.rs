#![feature(iter_array_chunks)]

use std::ops::BitAnd;

fn to_priority(c: u8) -> u8 {
    match c {
        c @ b'a'..=b'z' => c - b'a' + 1,
        c @ b'A'..=b'Z' => c - b'A' + 27,
        _ => 0,
    }
}

fn union(acc: u64, bit: u8) -> u64 {
    assert!(0 < bit && bit < 53);
    acc | (1 << bit)
}

fn intersect<const N: usize>(arr: [&[u8]; N]) -> u32 {
    let result = arr
        .into_iter()
        .map(|bag| bag.iter().copied().map(to_priority).fold(0, union))
        .fold(u64::MAX, u64::bitand)
        .trailing_zeros();

    assert!(0 < result && result < 53);
    result
}

fn part_1(input: &str) -> u32 {
    input
        .lines()
        .map(str::as_bytes)
        .map(|l| l.split_at(l.len() / 2))
        .map(|(left, right)| intersect([left, right]))
        .sum()
}

fn part_2(input: &str) -> u32 {
    input
        .lines()
        .map(str::as_bytes)
        .array_chunks::<3>()
        .map(intersect)
        .sum()
}

fn main() {
    const INPUT: &str = include_str!("input/given.txt");

    let sum_1 = part_1(INPUT);
    let sum_2 = part_2(INPUT);

    println!("The sums are {sum_1} and {sum_2}.")
}

#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn example_works() {
        const INPUT: &str = include_str!("input/example.txt");

        let sum_1 = part_1(INPUT);
        let sum_2 = part_2(INPUT);

        assert_eq!(sum_1, 157);
        assert_eq!(sum_2, 70);
    }
}
