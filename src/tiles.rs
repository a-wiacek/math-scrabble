use num::rational::Ratio;
use num::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use std::fmt;

use crate::consts::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Digit {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
}

impl Digit {
    pub fn all() -> impl Iterator<Item = Digit> {
        vec![
            Digit::D0,
            Digit::D1,
            Digit::D2,
            Digit::D3,
            Digit::D4,
            Digit::D5,
            Digit::D6,
            Digit::D7,
            Digit::D8,
            Digit::D9,
        ]
        .into_iter()
    }

    pub fn value(&self) -> i64 {
        match self {
            Digit::D0 => 0,
            Digit::D1 => 1,
            Digit::D2 => 2,
            Digit::D3 => 3,
            Digit::D4 => 4,
            Digit::D5 => 5,
            Digit::D6 => 6,
            Digit::D7 => 7,
            Digit::D8 => 8,
            Digit::D9 => 9,
        }
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum MathOperation {
    Plus,
    Minus,
    Times,
    Div,
}

impl MathOperation {
    pub fn all() -> impl Iterator<Item = MathOperation> {
        vec![
            MathOperation::Plus,
            MathOperation::Minus,
            MathOperation::Times,
            MathOperation::Div,
        ]
        .into_iter()
    }

    pub fn precedence(&self) -> u8 {
        match self {
            MathOperation::Plus => 2,
            MathOperation::Minus => 2,
            MathOperation::Times => 1,
            MathOperation::Div => 1,
        }
    }

    pub fn eval(&self, a: Ratio<i64>, b: Ratio<i64>) -> Option<Ratio<i64>> {
        match self {
            MathOperation::Plus => a.checked_add(&b),
            MathOperation::Minus => a.checked_sub(&b),
            MathOperation::Times => a.checked_mul(&b),
            MathOperation::Div => a.checked_div(&b),
        }
    }
}

impl fmt::Display for MathOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathOperation::Plus => write!(f, "+"),
            MathOperation::Minus => write!(f, "-"),
            MathOperation::Times => write!(f, "*"),
            MathOperation::Div => write!(f, "/"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Digit(Digit),
    Op(MathOperation),
    EqualsSign,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Digit(d) => write!(f, "{}", d),
            Tile::Op(op) => write!(f, "{}", op),
            Tile::EqualsSign => write!(f, "="),
        }
    }
}

impl Tile {
    pub fn score(&self) -> u64 {
        match self {
            Tile::Digit(d) => match d {
                Digit::D0 | Digit::D1 | Digit::D2 | Digit::D3 | Digit::D4 => 1,
                _ => 2,
            },
            Tile::Op(op) => match op {
                MathOperation::Plus => 1,
                MathOperation::Minus => 2,
                MathOperation::Times => 3,
                MathOperation::Div => 4,
            },
            Tile::EqualsSign => 0,
        }
    }

    pub fn initial_bag_of_tiles() -> Vec<Tile> {
        // Tiles with equals sign are not included here.
        // They are given by need: at each point in game each player
        // must have exactly one equals sign.
        let mut all_tiles = Vec::new();
        for d in Digit::all() {
            for _ in 0..DIGITS_IN_BAG {
                all_tiles.push(Tile::Digit(d));
            }
        }
        for op in MathOperation::all() {
            for _ in 0..OPERATORS_IN_BAG {
                all_tiles.push(Tile::Op(op));
            }
        }
        all_tiles
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum TileStatus {
    Permanent,
    Temporary,
}

#[derive(Clone)]
pub struct TileOnBoard {
    content: Tile,
    status: TileStatus,
}

impl TileOnBoard {
    pub fn new(content: Tile) -> TileOnBoard {
        TileOnBoard {
            content,
            status: TileStatus::Temporary,
        }
    }

    pub fn content(&self) -> Tile {
        self.content.clone()
    }

    pub fn status(&self) -> TileStatus {
        self.status
    }

    pub fn mark_as_permanent(&mut self) {
        self.status = TileStatus::Permanent;
    }
}
