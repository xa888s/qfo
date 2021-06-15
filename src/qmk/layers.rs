use thiserror::Error;

// The maximum amount of layers that can possibly exist
pub const MAX_LAYERS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    // generated code thankfully :D
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
    Seventeen,
    Eighteen,
    Nineteen,
    Twenty,
    TwentyOne,
    TwentyTwo,
    TwentyThree,
    TwentyFour,
    TwentyFive,
    TwentySix,
    TwentySeven,
    TwentyEight,
    TwentyNine,
    Thirty,
    ThirtyOne,
}

const LAYER_LIST: [Layer; MAX_LAYERS] = [
    Layer::Zero,
    Layer::One,
    Layer::Two,
    Layer::Three,
    Layer::Four,
    Layer::Five,
    Layer::Six,
    Layer::Seven,
    Layer::Eight,
    Layer::Nine,
    Layer::Ten,
    Layer::Eleven,
    Layer::Twelve,
    Layer::Thirteen,
    Layer::Fourteen,
    Layer::Fifteen,
    Layer::Sixteen,
    Layer::Seventeen,
    Layer::Eighteen,
    Layer::Nineteen,
    Layer::Twenty,
    Layer::TwentyOne,
    Layer::TwentyTwo,
    Layer::TwentyThree,
    Layer::TwentyFour,
    Layer::TwentyFive,
    Layer::TwentySix,
    Layer::TwentySeven,
    Layer::TwentyEight,
    Layer::TwentyNine,
    Layer::Thirty,
    Layer::ThirtyOne,
];

impl Layer {
    pub fn iter() -> impl Iterator<Item = Layer> {
        LAYER_LIST.iter().copied()
    }
}

#[derive(Debug, Error)]
#[error("layer too big, (max: {}, actual: {0})", MAX_LAYERS)]
pub struct LayerParseError(u8);

use std::convert::TryFrom;
impl TryFrom<u8> for Layer {
    type Error = LayerParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        LAYER_LIST
            .get(value as usize)
            .copied()
            .ok_or(LayerParseError(value))
    }
}
