use itertools::Itertools;

const INPUT: &str = include_str!("inputs/given.txt");

fn main() {
    let mut sums = sum_batched_lined(INPUT).collect_vec();

    let result = sum_n_largest(&mut sums, 1);
    println!("The amount of calories carried by one elf is {result}.");

    let result = sum_n_largest(&mut sums, 3);
    println!("The amount of calories carried by three elfs is {result}.");
}

fn sum_batched_lined(input: &str) -> impl Iterator<Item = u32> + '_ {
    input
        .lines()
        .map(|line: &str| line.parse::<u32>().ok())
        .batching(|iter| iter.while_some().sum1())
}

fn sum_n_largest(slice: &mut [u32], n: usize) -> u32 {
    let start = slice.len() - n;
    slice.select_nth_unstable(start);
    slice[start..].iter().sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_example() {
        const INPUT: &str = include_str!("inputs/example.txt");

        let result = sum_batched_lined(INPUT);

        assert_eq!(result.collect_vec(), vec![6000, 4000, 11000, 24000, 10000])
    }

    #[test]
    fn iter_no_empty_line_at_end() {
        const INPUT: &str = "1000";

        let result = sum_batched_lined(INPUT);

        assert_eq!(result.collect_vec(), vec![1000])
    }
}
