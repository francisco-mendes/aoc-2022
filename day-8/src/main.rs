use std::{
    collections::HashSet,
    convert::identity,
    ops::ControlFlow,
};

use ndarray::{
    s,
    Array2,
    ArrayBase,
    ArrayView1,
    ArrayView2,
    Ix2,
    OwnedRepr,
};

fn main() {
    const INPUT: &str = include_str!("input/given.txt");

    let side_len = INPUT.lines().next().unwrap().len();
    let bytes: Vec<_> = INPUT
        .as_bytes()
        .iter()
        .copied()
        .filter(u8::is_ascii_digit)
        .map(|b| b - b'0')
        .collect();

    let map = Array2::<u8>::from_shape_vec((side_len, side_len), bytes).unwrap();

    let visible = count_visible_from_outside(side_len, map.view());
    println!("the number of visible trees is {}", visible.len());

    let (pos, score) = find_most_scenic_tree(side_len, map);
    println!("the most scenic tree is at {pos:?} with score {score}");
}

fn find_most_scenic_tree(
    side_len: usize,
    map: ArrayBase<OwnedRepr<u8>, Ix2>,
) -> ((usize, usize), usize) {
    let scores = Array2::from_shape_fn((side_len - 2, side_len - 2), |(i, j)| {
        let (i, j) = (i + 1, j + 1);
        let height = map[(i, j)];
        let above = count_scenic_line(height, map.slice(s![i, ..j; -1]));
        let below = count_scenic_line(height, map.slice(s![i, j + 1..]));
        let left = count_scenic_line(height, map.slice(s![..i; -1, j]));
        let right = count_scenic_line(height, map.slice(s![i + 1.., j]));

        above * below * left * right
    });

    scores
        .indexed_iter()
        .max_by_key(|(_, &v)| v)
        .map(|((i, j), &v)| ((i + 1, j + 1), v))
        .unwrap()
}

fn count_scenic_line(tree_height: u8, line_of_sight: ArrayView1<u8>) -> usize {
    let score = line_of_sight.into_iter().try_fold(0, |count, &height| {
        if height < tree_height {
            ControlFlow::Continue(count + 1)
        } else {
            ControlFlow::Break(count + 1)
        }
    });

    match score {
        ControlFlow::Continue(score) | ControlFlow::Break(score) => score,
    }
}

fn count_visible_from_outside(side_len: usize, map: ArrayView2<u8>) -> HashSet<(usize, usize)> {
    let mut visible = HashSet::with_capacity(map.len() / 8 + side_len * 4);

    let iter = (0..side_len).flat_map(|i| [(i, 0), (0, i), (i, side_len - 1), (side_len - 1, i)]);
    visible.extend(iter);

    for i in 1..side_len - 1 {
        insert_visible_from_outside(
            &mut visible,
            map.slice(s![i, ..side_len - 1]),
            identity,
            |index| (i, index),
        );

        insert_visible_from_outside(
            &mut visible,
            map.slice(s![i, 1..; -1]),
            |index| side_len - 1 - index,
            |index| (i, index),
        );

        insert_visible_from_outside(
            &mut visible,
            map.slice(s![..side_len - 1, i]),
            identity,
            |index| (index, i),
        );

        insert_visible_from_outside(
            &mut visible,
            map.slice(s![1..; -1, i]),
            |index| side_len - 1 - index,
            |index| (index, i),
        );
    }

    visible
}

fn insert_visible_from_outside(
    visible: &mut HashSet<(usize, usize)>,
    line: ArrayView1<u8>,
    adapt_index: impl Fn(usize) -> usize,
    into_grid_index: impl Fn(usize) -> (usize, usize),
) {
    let mut iter = line
        .iter()
        .copied()
        .enumerate()
        .map(|(index, value)| (adapt_index(index), value));

    let start = iter.next().unwrap();
    let iter = iter.scan(start, scan_taller).flatten().map(into_grid_index);
    visible.extend(iter);
}

fn scan_taller(prev: &mut (usize, u8), (index, height): (usize, u8)) -> Option<Option<usize>> {
    let max = prev.1;
    if height > max {
        *prev = (index, height);
        Some(Some(index))
    } else {
        Some(None)
    }
}
