pub mod auto;

mod cons;
mod biunify;

pub use self::cons::Constructor;

use std::ops;

pub trait TypeSystem {
    type Constructor: Constructor;
    type Symbol: auto::Symbol;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
