// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::error::Result;
use crate::iname::Iname;
use crate::packable::{Mule, Packable};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumString, Display, EnumIter)]
pub enum Keyword {
    #[strum(serialize = "UnreachableKeywordStart")]
    KeywordStart = 127,
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,

    // mathematical special forms
    //
    #[strum(serialize = "+")]
    Plus,
    #[strum(serialize = "-")]
    Minus,
    #[strum(serialize = "*")]
    Mult,
    #[strum(serialize = "/")]
    Divide,
    #[strum(serialize = "=")]
    Equal,
    #[strum(serialize = ">")]
    Gt,
    #[strum(serialize = "<")]
    Lt,

    // built-in keywords/special-forms
    //
    #[strum(serialize = "++")]
    VectorAppend,
    #[strum(serialize = "sqrt")]
    Sqrt,
    #[strum(serialize = "mod")]
    Mod,
    #[strum(serialize = "and")]
    And,
    #[strum(serialize = "or")]
    Or,
    #[strum(serialize = "not")]
    Not,
    #[strum(serialize = "define")]
    Define,
    #[strum(serialize = "fn")]
    Fn,
    #[strum(serialize = "if")]
    If,
    #[strum(serialize = "each")]
    Each,
    #[strum(serialize = "loop")]
    Loop,
    #[strum(serialize = "fence")]
    Fence,
    #[strum(serialize = "on-matrix-stack")]
    OnMatrixStack,
    #[strum(serialize = "setq")]
    Setq,
    #[strum(serialize = "address-of")]
    AddressOf,
    #[strum(serialize = "fn-call")]
    FnCall,
    #[strum(serialize = "quote")]
    Quote,

    // pre-defined globals
    //
    #[strum(serialize = "#vars")]
    HashVars,
    #[strum(serialize = "canvas/centre")]
    CanvasCentre,
    #[strum(serialize = "canvas/width")]
    CanvasWidth,
    #[strum(serialize = "canvas/height")]
    CanvasHeight,
    #[strum(serialize = "canvas/size")]
    CanvasSize,
    #[strum(serialize = "math/PI")]
    MathPi,
    #[strum(serialize = "math/TAU")]
    MathTau,

    // colour formats
    //
    #[strum(serialize = "RGB")]
    Rgb,
    #[strum(serialize = "HSL")]
    Hsl,
    #[strum(serialize = "HSLuv")]
    Hsluv,
    #[strum(serialize = "LAB")]
    Lab,
    #[strum(serialize = "HSV")]
    Hsv,

    // pre-defined colours
    //
    #[strum(serialize = "white")]
    White,
    #[strum(serialize = "black")]
    Black,
    #[strum(serialize = "red")]
    Red,
    #[strum(serialize = "green")]
    Green,
    #[strum(serialize = "blue")]
    Blue,
    #[strum(serialize = "yellow")]
    Yellow,
    #[strum(serialize = "magenta")]
    Magenta,
    #[strum(serialize = "cyan")]
    Cyan,

    // procedural colours
    //
    #[strum(serialize = "chrome")]
    Chrome,
    #[strum(serialize = "hotline-miami")]
    HotlineMiami,
    #[strum(serialize = "knight-rider")]
    KnightRider,
    #[strum(serialize = "mars")]
    Mars,
    #[strum(serialize = "rainbow")]
    Rainbow,
    #[strum(serialize = "robocop")]
    Robocop,
    #[strum(serialize = "transformers")]
    Transformers,
    #[strum(serialize = "col/procedural-fn-presets")] // globally mapped in preamble
    ColProceduralFnPresets,

    // brush types
    //
    #[strum(serialize = "brush/flat")]
    BrushFlat,
    #[strum(serialize = "brush/a")]
    BrushA,
    #[strum(serialize = "brush/b")]
    BrushB,
    #[strum(serialize = "brush/c")]
    BrushC,
    #[strum(serialize = "brush/d")]
    BrushD,
    #[strum(serialize = "brush/e")]
    BrushE,
    #[strum(serialize = "brush/f")]
    BrushF,
    #[strum(serialize = "brush/g")]
    BrushG,
    #[strum(serialize = "brush/*")] // globally mapped in preamble
    BrushAll,

    // interpolation
    //
    #[strum(serialize = "linear")]
    Linear,
    #[strum(serialize = "ease/quick")]
    EaseQuick,
    #[strum(serialize = "ease/slow-in")]
    EaseSlowIn,
    #[strum(serialize = "ease/slow-in-out")]
    EaseSlowInOut,
    #[strum(serialize = "ease/quadratic-in")]
    EaseQuadraticIn,
    #[strum(serialize = "ease/quadratic-out")]
    EaseQuadraticOut,
    #[strum(serialize = "ease/quadratic-in-out")]
    EaseQuadraticInOut,
    #[strum(serialize = "ease/cubic-in")]
    EaseCubicIn,
    #[strum(serialize = "ease/cubic-out")]
    EaseCubicOut,
    #[strum(serialize = "ease/cubic-in-out")]
    EaseCubicInOut,
    #[strum(serialize = "ease/quartic-in")]
    EaseQuarticIn,
    #[strum(serialize = "ease/quartic-out")]
    EaseQuarticOut,
    #[strum(serialize = "ease/quartic-in-out")]
    EaseQuarticInOut,
    #[strum(serialize = "ease/quintic-in")]
    EaseQuinticIn,
    #[strum(serialize = "ease/quintic-out")]
    EaseQuinticOut,
    #[strum(serialize = "ease/quintic-in-out")]
    EaseQuinticInOut,
    #[strum(serialize = "ease/sin-in")]
    EaseSinIn,
    #[strum(serialize = "ease/sin-out")]
    EaseSinOut,
    #[strum(serialize = "ease/sin-in-out")]
    EaseSinInOut,
    #[strum(serialize = "ease/circular-in")]
    EaseCircularIn,
    #[strum(serialize = "ease/circular-out")]
    EaseCircularOut,
    #[strum(serialize = "ease/circular-in-out")]
    EaseCircularInOut,
    #[strum(serialize = "ease/exponential-in")]
    EaseExponentialIn,
    #[strum(serialize = "ease/exponential-out")]
    EaseExponentialOut,
    #[strum(serialize = "ease/exponential-in-out")]
    EaseExponentialInOut,
    #[strum(serialize = "ease/elastic-in")]
    EaseElasticIn,
    #[strum(serialize = "ease/elastic-out")]
    EaseElasticOut,
    #[strum(serialize = "ease/elastic-in-out")]
    EaseElasticInOut,
    #[strum(serialize = "ease/back-in")]
    EaseBackIn,
    #[strum(serialize = "ease/back-out")]
    EaseBackOut,
    #[strum(serialize = "ease/back-in-out")]
    EaseBackInOut,
    #[strum(serialize = "ease/bounce-in")]
    EaseBounceIn,
    #[strum(serialize = "ease/bounce-out")]
    EaseBounceOut,
    #[strum(serialize = "ease/bounce-in-out")]
    EaseBounceInOut,

    #[strum(serialize = "ease/*")] // globally mapped in preamble
    EaseAll,

    // common parameter labels
    //
    #[strum(serialize = "a")]
    A,
    #[strum(serialize = "b")]
    B,
    #[strum(serialize = "c")]
    C,
    #[strum(serialize = "d")]
    D,
    #[strum(serialize = "g")]
    G,
    #[strum(serialize = "h")]
    H,
    #[strum(serialize = "l")]
    L,
    #[strum(serialize = "n")]
    N,
    #[strum(serialize = "r")]
    R,
    #[strum(serialize = "s")]
    S,
    #[strum(serialize = "t")]
    T,
    #[strum(serialize = "u")]
    U,
    #[strum(serialize = "v")]
    V,
    #[strum(serialize = "x")]
    X,
    #[strum(serialize = "y")]
    Y,
    #[strum(serialize = "z")]
    Z,
    #[strum(serialize = "alpha")]
    Alpha,
    #[strum(serialize = "amplitude")]
    Amplitude,
    #[strum(serialize = "angle")]
    Angle,
    #[strum(serialize = "angle-end")]
    AngleEnd,
    #[strum(serialize = "angle-start")]
    AngleStart,
    #[strum(serialize = "brightness")]
    Brightness,
    #[strum(serialize = "brush")]
    Brush,
    #[strum(serialize = "brush-subtype")]
    BrushSubtype,
    #[strum(serialize = "by")]
    By,
    #[strum(serialize = "clamping")]
    Clamping,
    #[strum(serialize = "colour")]
    Colour,
    #[strum(serialize = "colour-volatility")]
    ColourVolatility,
    #[strum(serialize = "colours")]
    Colours,
    #[strum(serialize = "contrast")]
    Contrast,
    #[strum(serialize = "coords")]
    Coords,
    #[strum(serialize = "copies")]
    Copies,
    #[strum(serialize = "copy")]
    Copy,
    #[strum(serialize = "default-colour")]
    DefaultColour,
    #[strum(serialize = "direction")]
    Direction,
    #[strum(serialize = "distance")]
    Distance,
    #[strum(serialize = "format")]
    Format,
    #[strum(serialize = "frequency")]
    Frequency,
    #[strum(serialize = "from")]
    From,
    #[strum(serialize = "from-colour")]
    FromColour,
    #[strum(serialize = "height")]
    Height,
    #[strum(serialize = "inc")]
    Inc,
    #[strum(serialize = "index")]
    Index,
    #[strum(serialize = "inner-colour")]
    InnerColour,
    #[strum(serialize = "inner-height")]
    InnerHeight,
    #[strum(serialize = "inner-radius")]
    InnerRadius,
    #[strum(serialize = "inner-width")]
    InnerWidth,
    #[strum(serialize = "invert")]
    Invert,
    #[strum(serialize = "iterations")]
    Iterations,
    #[strum(serialize = "linear-colour-space")]
    LinearColourSpace,
    #[strum(serialize = "line-width")]
    LineWidth,
    #[strum(serialize = "line-width-end")]
    LineWidthEnd,
    #[strum(serialize = "line-width-mapping")]
    LineWidthMapping,
    #[strum(serialize = "line-width-start")]
    LineWidthStart,
    #[strum(serialize = "mapping")]
    Mapping,
    #[strum(serialize = "max")]
    Max,
    #[strum(serialize = "min")]
    Min,
    #[strum(serialize = "num")]
    Num,
    #[strum(serialize = "outer-colour")]
    OuterColour,
    #[strum(serialize = "outer-radius")]
    OuterRadius,
    #[strum(serialize = "overlap")]
    Overlap,
    #[strum(serialize = "point")]
    Point,
    #[strum(serialize = "position")]
    Position,
    #[strum(serialize = "preset")]
    Preset,
    #[strum(serialize = "radius")]
    Radius,
    #[strum(serialize = "saturation")]
    Saturation,
    #[strum(serialize = "scalar")]
    Scalar,
    #[strum(serialize = "scalars")]
    Scalars,
    #[strum(serialize = "seed")]
    Seed,
    #[strum(serialize = "shuffle-seed")]
    ShuffleSeed,
    #[strum(serialize = "steps")]
    Steps,
    #[strum(serialize = "stroke-line-width-end")]
    StrokeLineWidthEnd,
    #[strum(serialize = "stroke-line-width-start")]
    StrokeLineWidthStart,
    #[strum(serialize = "stroke-noise")]
    StrokeNoise,
    #[strum(serialize = "stroke-tessellation")]
    StrokeTessellation,
    #[strum(serialize = "t-end")]
    TEnd,
    #[strum(serialize = "t-start")]
    TStart,
    #[strum(serialize = "tessellation")]
    Tessellation,
    #[strum(serialize = "transform-position")]
    TransformPosition,
    #[strum(serialize = "to")]
    To,
    #[strum(serialize = "to-colour")]
    ToColour,
    #[strum(serialize = "upto")]
    Upto,
    #[strum(serialize = "value")]
    Value,
    #[strum(serialize = "vec1")]
    Vec1,
    #[strum(serialize = "vec2")]
    Vec2,
    #[strum(serialize = "vector")]
    Vector,
    #[strum(serialize = "volatility")]
    Volatility,
    #[strum(serialize = "width")]
    Width,
    #[strum(serialize = "worldspace")]
    WorldSpace,

    #[strum(serialize = "UnreachableKeywordEnd")]
    KeywordEnd,
}

impl Packable for Keyword {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_label(cursor, &self.to_string());

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<Keyword>()?;

        Ok((res, &cursor[ns..]))
    }
}

pub fn name_to_keyword_hash() -> HashMap<Iname, Keyword> {
    let mut hm: HashMap<Iname, Keyword> = HashMap::new();

    for kw in Keyword::iter() {
        hm.insert(Iname::from(kw), kw);
    }

    hm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_enums() {
        assert_eq!(Keyword::False as i32, 128);
        assert_eq!(Keyword::True as i32, 129);
    }

    #[test]
    fn test_keyword_pack() {
        let mut res: String = "".into();
        Keyword::Volatility.pack(&mut res).unwrap();
        assert_eq!("volatility", res);
    }

    #[test]
    fn test_keyword_unpack() {
        let (res, _rem) = Keyword::unpack("volatility").unwrap();
        assert_eq!(res, Keyword::Volatility);
    }
}
