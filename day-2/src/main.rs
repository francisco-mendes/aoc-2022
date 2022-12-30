use strategy::{
    FixedMatch,
    Roshambo,
    Strategy,
};
use Outcome::*;
use Shape::*;

mod strategy;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Outcome {
    Lose,
    Draw,
    Win,
}

impl TryFrom<u8> for Outcome {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'X' => Ok(Lose),
            b'Y' => Ok(Draw),
            b'Z' => Ok(Win),
            _ => Err(()),
        }
    }
}

impl Outcome {
    pub fn score(self) -> u32 {
        match self {
            Lose => 0,
            Draw => 3,
            Win => 6,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Shape {
    Rock,
    Paper,
    Scissors,
}

impl TryFrom<u8> for Shape {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'A' | b'X' => Ok(Rock),
            b'B' | b'Y' => Ok(Paper),
            b'C' | b'Z' => Ok(Scissors),
            _ => Err(()),
        }
    }
}

impl Shape {
    pub fn score(self) -> u32 {
        match self {
            Rock => 1,
            Paper => 2,
            Scissors => 3,
        }
    }

    pub fn solve_for(self, result: Outcome) -> Self {
        match (self, result) {
            (Rock, Draw) | (Scissors, Win) | (Paper, Lose) => Rock,
            (Paper, Draw) | (Rock, Win) | (Scissors, Lose) => Paper,
            (Scissors, Draw) | (Paper, Win) | (Rock, Lose) => Scissors,
        }
    }
}

pub fn compute_score<S>(input: &str) -> S::Out
where
    S: Strategy,
{
    input
        .lines()
        .map(str::as_bytes)
        .map(|c| (c[0], c[2]))
        .filter_map(S::parse_match)
        .map(S::match_score)
        .fold(S::Out::default(), S::reduce)
}

fn main() {
    const INPUT: &str = include_str!("inputs/given.txt");

    let (first, second) = compute_score::<(Roshambo, FixedMatch)>(INPUT);
    println!("The expected score for roshambo is {first}");
    println!("The expected score for fixed matches is {second}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_round() {
        const INPUT: &str = "A X";

        let total_score = compute_score::<(Roshambo, FixedMatch)>(INPUT);

        let roshambo = Draw.score() + Rock.score();
        let fixed = Lose.score() + Scissors.score();
        assert_eq!(total_score, (roshambo, fixed))
    }

    #[test]
    fn example_works_one() {
        const INPUT: &str = include_str!("inputs/example.txt");

        let total_score = compute_score::<Roshambo>(INPUT);

        assert_eq!(total_score, 15)
    }

    #[test]
    fn example_works_two() {
        const INPUT: &str = include_str!("inputs/example.txt");

        let total_score = compute_score::<FixedMatch>(INPUT);

        assert_eq!(total_score, 12)
    }
}
