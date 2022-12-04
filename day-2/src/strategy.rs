use crate::Outcome::{Draw, Lose, Win};
use crate::{Outcome, Shape};
use crate::Shape::{Paper, Rock, Scissors};

pub trait Strategy {
    type Input;
    type Out: Default;

    fn parse_match(line: (u8, u8)) -> Option<Self::Input>;

    fn match_score(input: Self::Input) -> Self::Out;

    fn reduce(a: Self::Out, b: Self::Out) -> Self::Out;
}

pub struct Roshambo;

impl Strategy for Roshambo {
    type Input = (Shape, Shape);
    type Out = u32;

    fn parse_match((this, other): (u8, u8)) -> Option<Self::Input> {
        let self_play = Shape::try_from(other).ok()?;
        let other_play = Shape::try_from(this).ok()?;

        Some((self_play, other_play))
    }

    fn match_score((self_play, other_play): Self::Input) -> Self::Out {
        let result = match (self_play, other_play) {
            (Paper, Rock) | (Scissors, Paper) | (Rock, Scissors) => Win,
            (a, b) if a == b => Draw,
            _ => Lose,
        };

        self_play.score() + result.score()
    }

    fn reduce(a: Self::Out, b: Self::Out) -> Self::Out {
        a + b
    }
}

pub struct FixedMatch;

impl Strategy for FixedMatch {
    type Input = (Outcome, Shape);
    type Out = u32;

    fn parse_match((this, other): (u8, u8)) -> Option<Self::Input> {
        let result = Outcome::try_from(other).ok()?;
        let other_play = Shape::try_from(this).ok()?;

        Some((result, other_play))
    }

    fn match_score((result, other_play): Self::Input) -> Self::Out {
        let self_play = other_play.solve_for(result);
        self_play.score() + result.score()
    }

    fn reduce(a: Self::Out, b: Self::Out) -> Self::Out {
        a + b
    }
}

impl<A, B> Strategy for (A, B)
where
    A: Strategy,
    B: Strategy,
{
    type Input = (A::Input, B::Input);
    type Out = (A::Out, B::Out);

    fn parse_match(line: (u8, u8)) -> Option<Self::Input> {
        Some((A::parse_match(line)?, B::parse_match(line)?))
    }

    fn match_score((a, b): Self::Input) -> Self::Out {
        (A::match_score(a), B::match_score(b))
    }

    fn reduce((a1, b1): Self::Out, (a2, b2): Self::Out) -> Self::Out {
        (A::reduce(a1, a2), B::reduce(b1, b2))
    }
}
