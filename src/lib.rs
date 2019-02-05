pub mod auto;
pub mod polar;

mod biunify;
mod subsume;
mod cons;
#[cfg(test)]
mod tests;

pub use self::cons::Constructor;

use std::fmt::Debug;
use std::ops;

pub trait TypeSystem {
    type Constructor: Constructor + Debug;
    type Symbol: auto::Symbol + Debug;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Polarity {
    Neg = -1,
    Pos = 1,
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
