pub mod auto;
pub mod polar;

mod biunify;
mod cons;
mod subsume;
#[cfg(test)]
mod tests;

pub use self::cons::{Constructor, Label};

pub(crate) use self::cons::ConstructorSet;

use std::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Polarity {
    Neg = -1,
    Pos = 1,
}

impl Polarity {
    pub(crate) fn flip<T>(self, a: T, b: T) -> (T, T) {
        match self {
            Polarity::Pos => (a, b),
            Polarity::Neg => (b, a),
        }
    }
}

impl ops::Neg for Polarity {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Polarity::Neg => Polarity::Pos,
            Polarity::Pos => Polarity::Neg,
        }
    }
}

impl ops::Mul for Polarity {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match self {
            Polarity::Neg => -other,
            Polarity::Pos => other,
        }
    }
}
