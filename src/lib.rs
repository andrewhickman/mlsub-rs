pub mod auto;
pub mod cons;
pub mod polar;

mod biunify;
mod subsume;
#[cfg(test)]
mod tests;

pub use self::biunify::{Error as BiunifyError, Result as BiunifyResult};
pub use self::cons::{Constructor, ConstructorSet, Label};

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
