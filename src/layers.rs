use thiserror::Error;

// The maximum amount of layers that can possibly exist
pub const MAX_LAYERS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum Layer {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Eleven = 11,
    Twelve = 12,
    Thirteen = 13,
    Fourteen = 14,
    Fifteen = 15,
}

#[derive(Debug, Error)]
#[error("failed to parse into layer")]
pub struct LayerParseError;

use std::convert::TryFrom;
impl TryFrom<usize> for Layer {
    type Error = LayerParseError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use Layer::*;

        Ok(match value {
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            9 => Nine,
            10 => Ten,
            11 => Eleven,
            12 => Twelve,
            13 => Thirteen,
            14 => Fourteen,
            15 => Fifteen,
            _ => return Err(LayerParseError),
        })
    }
}
