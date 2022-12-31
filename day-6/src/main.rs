#![feature(portable_simd)]

use std::{
    array,
    cmp::max,
    convert::identity,
    ops::{
        BitXor,
        Not,
    },
    simd::{
        Mask,
        Simd,
        SimdOrd,
        SimdPartialEq,
        SimdPartialOrd,
        SimdUint,
    },
};

/// Computes the minimum increment before another possible unique sequence.
///
/// For an arbitrary sequence with four values 'abcd', the increment is the number of values that need to be skipped over.
/// The increment depends on which values are repeated in the sequence, which can be calculated from the following table:
///
/// |       | **a** | **b** | **c** | **d** |
/// |:-----:|:-----:|:-----:|:-----:|:-----:|
/// | **a** |   0   |   1   |   1   |   1   |
/// | **b** |   1   |   0   |   2   |   2   |
/// | **c** |   1   |   2   |   0   |   3   |
/// | **d** |   1   |   2   |   3   |   0   |
///
/// # Examples
/// For a sequence '1234', all elements are unique, so we finish with an increment of 0.
///
/// For a sequence '1123', 'a' and 'b' are equal and we have to skip over the first element, so the increment is 1.
///
/// For a sequence '1223', 'b' and 'c' are equal and we have to skip over the first two elements, so the increment is 2.
///
/// For a sequence '1233', 'c' and 'd' are equal and we have to skip over the first three elements, so the increment is 3.
///
///
fn compute_increment_4(arr: &[u8; 4]) -> usize {
    let mask = Mask::splat(true);
    let default = Simd::splat(0);

    let cols = Simd::from_array([0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3]);
    let rows = Simd::from_array([0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    let incr = Simd::from_array([0, 1, 1, 1, 1, 0, 2, 2, 1, 2, 0, 3, 1, 2, 3, 0]);

    let cols = unsafe { Simd::gather_select_unchecked(arr, mask, cols, default) };
    let rows = unsafe { Simd::gather_select_unchecked(arr, mask, rows, default) };

    let max_incr = cols.simd_eq(rows).select(incr, default).reduce_max();

    usize::from(max_incr)
}

fn compute_increment_14(arr: &[u8; 14]) -> usize {
    let indices = Simd::from_array(array::from_fn(identity));
    let pick = indices.simd_lt(Simd::splat(14));
    let cols = unsafe { Simd::gather_select_unchecked(arr, pick, indices, Simd::splat(0)) };

    let increments: Simd<u8, 16> = indices.cast();
    let increments: Simd<u8, 16> = increments + Simd::splat(1);
    let mut result = 0;
    for (index, val) in arr.iter().copied().enumerate().rev() {
        let row_index = Simd::splat((index + 1) as u8);
        let increments = increments
            .simd_eq(row_index)
            .bitxor(pick.not().cast())
            .select(Simd::splat(0), increments)
            .simd_min(row_index);

        let row = Simd::splat(val);
        let incr = cols
            .simd_eq(row)
            .select(increments, Simd::splat(0))
            .reduce_max();

        result = max(result, usize::from(incr));
    }

    result
}

pub fn find_post_unique_index<const N: usize>(
    input: &str,
    compute_increment: impl Fn(&[u8; N]) -> usize,
) -> Option<usize> {
    assert_ne!(N, 0);

    let mut index = 0;
    let iter = input.as_bytes();

    while iter.len().saturating_sub(index) > N {
        let arr: [u8; N] = array::from_fn(|i| unsafe { *iter.get_unchecked(index + i) });

        let incr = compute_increment(&arr);
        index += incr;
        if incr == 0 {
            return Some(index + N);
        }
    }

    None
}

fn main() {
    const INPUT: &str = include_str!("input/given.txt");

    let start = find_post_unique_index(INPUT, compute_increment_4).unwrap();
    let msg = find_post_unique_index(INPUT, compute_increment_14).unwrap();

    println!("The first packet index is {start}",);
    println!("The first message starts at {msg}",);
}

mod test {
    use yare::parameterized;

    use super::*;

    #[parameterized(
        unique = { b"1234", 0 },
        a_b_eq = { b"1123", 1 },
        a_c_eq = { b"1213", 1 },
        a_d_eq = { b"1231", 1 },
        b_c_eq = { b"1223", 2 },
        b_d_eq = { b"1232", 2 },
        c_d_eq = { b"1233", 3 },
        abc_eq = { b"1112", 2 },
        abd_eq = { b"1121", 2 },
        acd_eq = { b"1211", 3 },
        bcd_eq = { b"1222", 3 },
        all_eq = { b"1111", 3 },
    )]
    fn increment_cases_4(arr: &[u8; 4], expected: usize) {
        let incr = compute_increment_4(arr);

        assert_eq!(incr, expected);
    }

    #[parameterized(
        unique       = { b"1234abcdefghij", 0 },
        first_a_b_eq = { b"1123abcdefghij", 1 },
        first_a_c_eq = { b"1213abcdefghij", 1 },
        first_a_d_eq = { b"1231abcdefghij", 1 },
        first_b_c_eq = { b"1223abcdefghij", 2 },
        first_b_d_eq = { b"1232abcdefghij", 2 },
        first_c_d_eq = { b"1233abcdefghij", 3 },
        first_abc_eq = { b"1112abcdefghij", 2 },
        first_abd_eq = { b"1121abcdefghij", 2 },
        first_acd_eq = { b"1211abcdefghij", 3 },
        first_bcd_eq = { b"1222abcdefghij", 3 },
        first_all_eq = { b"1111abcdefghij", 3 },
        last_a_b_eq  = { b"abcdefghij1123", 11 },
        last_a_c_eq  = { b"abcdefghij1213", 11 },
        last_a_d_eq  = { b"abcdefghij1231", 11 },
        last_b_c_eq  = { b"abcdefghij1223", 12 },
        last_b_d_eq  = { b"abcdefghij1232", 12 },
        last_c_d_eq  = { b"abcdefghij1233", 13 },
        last_abc_eq  = { b"abcdefghij1112", 12 },
        last_abd_eq  = { b"abcdefghij1121", 12 },
        last_acd_eq  = { b"abcdefghij1211", 13 },
        last_bcd_eq  = { b"abcdefghij1222", 13 },
        last_all_eq  = { b"abcdefghij1111", 13 },
    )]
    fn increment_cases_14_first_four(arr: &[u8; 14], expected: usize) {
        let incr = compute_increment_14(arr);

        assert_eq!(incr, expected);
    }

    #[parameterized(
        empty     = { "" },
        small     = { "123" },
        no_unique = { "123123" },
    )]
    fn no_start(input: &str) {
        let actual = find_post_unique_index(input, compute_increment_4);

        assert_eq!(actual, None);
    }

    #[parameterized(
        empty     = { "" },
        small     = { "12345"},
        no_unique = { "12345123451234512345" },
    )]
    fn no_message(input: &str) {
        let actual = find_post_unique_index(input, compute_increment_14);

        assert_eq!(actual, None);
    }

    #[parameterized(
        example_0 = { "mjqjpqmgbljsphdztnvjfqwrcgsmlb",    Some(7) },
        example_1 = { "bvwbjplbgvbhsrlpgdmjqwftvncz",      Some(5) },
        example_2 = { "nppdvjthqldpwncqszvftbrmjlhg",      Some(6) },
        example_3 = { "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", Some(10) },
        example_4 = { "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw",  Some(11) },
    )]
    fn examples_packet(input: &str, expected: Option<usize>) {
        let actual = find_post_unique_index(input, compute_increment_4);

        assert_eq!(actual, expected);
    }

    #[parameterized(
        example_0 = { "mjqjpqmgbljsphdztnvjfqwrcgsmlb",    Some(19) },
        example_1 = { "bvwbjplbgvbhsrlpgdmjqwftvncz",      Some(23) },
        example_2 = { "nppdvjthqldpwncqszvftbrmjlhg",      Some(23) },
        example_3 = { "nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", Some(29) },
        example_4 = { "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw",  Some(26) },
    )]
    fn examples_message(input: &str, expected: Option<usize>) {
        let actual = find_post_unique_index(input, compute_increment_14);

        assert_eq!(actual, expected);
    }
}
