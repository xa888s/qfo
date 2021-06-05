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

impl Layer {
    pub fn iter() -> impl Iterator<Item = Layer> {
        use Layer::*;
        [
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
        ]
        .iter()
        .copied()
    }
}
