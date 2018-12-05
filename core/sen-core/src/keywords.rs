// Copyright (C) 2018 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;

// start at MAX_WORD_LOOKUPS

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Keyword {
    False = 128,
    True = 129,

    // mathematical special forms
    //
    Plus = 130,
    Minus = 131,
    Mult = 132,
    Divide = 133,
    Equal = 134,
    Gt = 135,
    Lt = 136,

    // built-in keywords/special-forms
    //
    VectorAppend = 137,
    Sqrt = 138,
    Mod = 139,
    And = 140,
    Or = 141,
    Not = 142,
    Define = 143,
    Fn = 144,
    If = 145,
    Each = 146,
    Loop = 147,
    Fence = 148,
    OnMatrixStack = 149,
    Setq = 150,
    AddressOf = 151,
    FnCall = 152,
    Quote = 153,

    // pre-defined globals
    //
    HashVars = 154,
    CanvasWidth = 155,
    CanvasHeight = 156,
    MathTau = 157,

    // colour formats
    //
    Rgb = 158,
    Hsl = 159,
    Hsluv = 160,
    Lab = 161,
    Hsv = 162,

    // pre-defined colours
    //
    White = 163,
    Black = 164,
    Red = 165,
    Green = 166,
    Blue = 167,
    Yellow = 168,
    Magenta = 169,
    Cyan = 170,

    // procedural colours
    //
    Chrome = 171,
    HotlineMiami = 172,
    KnightRider = 173,
    Mars = 174,
    Rainbow = 175,
    Robocop = 176,
    Transformers = 177,
    ColProceduralFnPresets = 178,

    // brush types
    //
    BrushFlat = 179,
    BrushA = 180,
    BrushB = 181,
    BrushC = 182,
    BrushD = 183,
    BrushE = 184,
    BrushF = 185,
    BrushG = 186,

    // interpolation
    //
    Linear = 187,
    EaseQuick = 188,
    EaseSlowIn = 189,
    EaseSlowInOut = 190,
    EaseQuadraticIn = 191,
    EaseQuadraticOut = 192,
    EaseQuadraticInOut = 193,
    EaseCubicIn = 194,
    EaseCubicOut = 195,
    EaseCubicInOut = 196,
    EaseQuarticIn = 197,
    EaseQuarticOut = 198,
    EaseQuarticInOut = 199,
    EaseQuinticIn = 200,
    EaseQuinticOut = 201,
    EaseQuinticInOut = 202,
    EaseSinIn = 203,
    EaseSinOut = 204,
    EaseSinInOut = 205,
    EaseCircularIn = 206,
    EaseCircularOut = 207,
    EaseCircularInOut = 208,
    EaseExponentialIn = 209,
    EaseExponentialOut = 210,
    EaseExponentialInOut = 211,
    EaseElasticIn = 212,
    EaseElasticOut = 213,
    EaseElasticInOut = 214,
    EaseBackIn = 215,
    EaseBackOut = 216,
    EaseBackInOut = 217,
    EaseBounceIn = 218,
    EaseBounceOut = 219,
    EaseBounceInOut = 220,

    EasePresets = 221,

    // common parameter labels
    //
    A = 222,
    B = 223,
    C = 224,
    D = 225,
    G = 226,
    H = 227,
    L = 228,
    N = 229,
    R = 230,
    S = 231,
    T = 232,
    V = 233,
    X = 234,
    Y = 235,
    Z = 236,
    Alpha = 237,
    Amplitude = 238,
    Angle = 239,
    AngleEnd = 240,
    AngleStart = 241,
    Brush = 242,
    BrushSubtype = 243,
    By = 244,
    Clamping = 245,
    Colour = 246,
    ColourVolatility = 247,
    Colours = 248,
    Coords = 249,
    Copies = 250,
    Copy = 251,
    Direction = 252,
    Distance = 253,
    Format = 254,
    Frequency = 255,
    From = 256,
    FromColour = 257,
    GenInitial = 258,
    Height = 259,
    Inc = 260,
    InnerHeight = 261,
    InnerWidth = 262,
    Iterations = 263,
    LineWidth = 264,
    LineWidthEnd = 265,
    LineWidthMapping = 266,
    LineWidthStart = 267,
    Mapping = 268,
    Max = 269,
    Min = 270,
    Num = 271,
    Overlap = 272,
    Point = 273,
    Position = 274,
    Preset = 275,
    Radius = 276,
    Scalar = 277,
    Seed = 278,
    Steps = 279,
    StrokeLineWidthEnd = 280,
    StrokeLineWidthStart = 281,
    StrokeNoise = 282,
    StrokeTessellation = 283,
    TEnd = 284,
    TStart = 285,
    Tessellation = 286,
    TransformPosition = 287,
    To = 288,
    ToColour = 289,
    Upto = 290,
    Value = 291,
    Vec1 = 292,
    Vec2 = 293,
    Vector = 294,
    Volatility = 295,
    Width = 296,
}

pub fn string_to_i32_hash() -> HashMap<String, i32> {
    let mut hm: HashMap<String, i32> = HashMap::new();

    let names = [
        (Keyword::False, "false"),
        (Keyword::True, "true"),
        (Keyword::Plus, "+"),
        (Keyword::Minus, "-"),
        (Keyword::Mult, "*"),
        (Keyword::Divide, "/"),
        (Keyword::Equal, "="),
        (Keyword::Gt, ">"),
        (Keyword::Lt, "<"),
        (Keyword::VectorAppend, "++"),
        (Keyword::Sqrt, "sqrt"),
        (Keyword::Mod, "mod"),
        (Keyword::And, "and"),
        (Keyword::Or, "or"),
        (Keyword::Not, "not"),
        (Keyword::Define, "define"),
        (Keyword::Fn, "fn"),
        (Keyword::If, "if"),
        (Keyword::Each, "each"),
        (Keyword::Loop, "loop"),
        (Keyword::Fence, "fence"),
        (Keyword::OnMatrixStack, "on-matrix-stack"),
        (Keyword::Setq, "setq"),
        (Keyword::AddressOf, "address-of"),
        (Keyword::FnCall, "fn-call"),
        (Keyword::Quote, "quote"),
        (Keyword::HashVars, "#vars"),
        (Keyword::CanvasWidth, "canvas/width"),
        (Keyword::CanvasHeight, "canvas/height"),
        (Keyword::MathTau, "math/TAU"),
        (Keyword::Rgb, "RGB"),
        (Keyword::Hsl, "HSL"),
        (Keyword::Hsluv, "HSLuv"),
        (Keyword::Lab, "LAB"),
        (Keyword::Hsv, "HSV"),
        (Keyword::White, "white"),
        (Keyword::Black, "black"),
        (Keyword::Red, "red"),
        (Keyword::Green, "green"),
        (Keyword::Blue, "blue"),
        (Keyword::Yellow, "yellow"),
        (Keyword::Magenta, "magenta"),
        (Keyword::Cyan, "cyan"),
        (Keyword::Chrome, "chrome"),
        (Keyword::HotlineMiami, "hotline-miami"),
        (Keyword::KnightRider, "knight-rider"),
        (Keyword::Mars, "mars"),
        (Keyword::Rainbow, "rainbow"),
        (Keyword::Robocop, "robocop"),
        (Keyword::Transformers, "transformers"),
        (Keyword::ColProceduralFnPresets, "col/procedural-fn-presets"),
        (Keyword::BrushFlat, "brush-flat"),
        (Keyword::BrushA, "brush-a"),
        (Keyword::BrushB, "brush-b"),
        (Keyword::BrushC, "brush-c"),
        (Keyword::BrushD, "brush-d"),
        (Keyword::BrushE, "brush-e"),
        (Keyword::BrushF, "brush-f"),
        (Keyword::BrushG, "brush-g"),
        (Keyword::Linear, "linear"),
        (Keyword::EaseQuick, "ease/quick"),
        (Keyword::EaseSlowIn, "ease/slow-in"),
        (Keyword::EaseSlowInOut, "ease/slow-in-out"),
        (Keyword::EaseQuadraticIn, "ease/quadratic-in"),
        (Keyword::EaseQuadraticOut, "ease/quadratic-out"),
        (Keyword::EaseQuadraticInOut, "ease/quadratic-in-out"),
        (Keyword::EaseCubicIn, "ease/cubic-in"),
        (Keyword::EaseCubicOut, "ease/cubic-out"),
        (Keyword::EaseCubicInOut, "ease/cubic-in-out"),
        (Keyword::EaseQuarticIn, "ease/quartic-in"),
        (Keyword::EaseQuarticOut, "ease/quartic-out"),
        (Keyword::EaseQuarticInOut, "ease/quartic-in-out"),
        (Keyword::EaseQuinticIn, "ease/quintic-in"),
        (Keyword::EaseQuinticOut, "ease/quintic-out"),
        (Keyword::EaseQuinticInOut, "ease/quintic-in-out"),
        (Keyword::EaseSinIn, "ease/sin-in"),
        (Keyword::EaseSinOut, "ease/sin-out"),
        (Keyword::EaseSinInOut, "ease/sin-in-out"),
        (Keyword::EaseCircularIn, "ease/circular-in"),
        (Keyword::EaseCircularOut, "ease/circular-out"),
        (Keyword::EaseCircularInOut, "ease/circular-in-out"),
        (Keyword::EaseExponentialIn, "ease/exponential-in"),
        (Keyword::EaseExponentialOut, "ease/exponential-out"),
        (Keyword::EaseExponentialInOut, "ease/exponential-in-out"),
        (Keyword::EaseElasticIn, "ease/elastic-in"),
        (Keyword::EaseElasticOut, "ease/elastic-out"),
        (Keyword::EaseElasticInOut, "ease/elastic-in-out"),
        (Keyword::EaseBackIn, "ease/back-in"),
        (Keyword::EaseBackOut, "ease/back-out"),
        (Keyword::EaseBackInOut, "ease/back-in-out"),
        (Keyword::EaseBounceIn, "ease/bounce-in"),
        (Keyword::EaseBounceOut, "ease/bounce-out"),
        (Keyword::EaseBounceInOut, "ease/bounce-in-out"),
        (Keyword::EasePresets, "ease/presets"),
        (Keyword::A, "a"),
        (Keyword::B, "b"),
        (Keyword::C, "c"),
        (Keyword::D, "d"),
        (Keyword::G, "g"),
        (Keyword::H, "h"),
        (Keyword::L, "l"),
        (Keyword::N, "n"),
        (Keyword::R, "r"),
        (Keyword::S, "s"),
        (Keyword::T, "t"),
        (Keyword::V, "v"),
        (Keyword::X, "x"),
        (Keyword::Y, "y"),
        (Keyword::Z, "z"),
        (Keyword::Alpha, "alpha"),
        (Keyword::Amplitude, "amplitude"),
        (Keyword::Angle, "angle"),
        (Keyword::AngleEnd, "angle-end"),
        (Keyword::AngleStart, "angle-start"),
        (Keyword::Brush, "brush"),
        (Keyword::BrushSubtype, "brush-subtype"),
        (Keyword::By, "by"),
        (Keyword::Clamping, "clamping"),
        (Keyword::Colour, "colour"),
        (Keyword::ColourVolatility, "colour-volatility"),
        (Keyword::Colours, "colours"),
        (Keyword::Coords, "coords"),
        (Keyword::Copies, "copies"),
        (Keyword::Copy, "copy"),
        (Keyword::Direction, "direction"),
        (Keyword::Distance, "distance"),
        (Keyword::Format, "format"),
        (Keyword::Frequency, "frequency"),
        (Keyword::From, "from"),
        (Keyword::FromColour, "from-colour"),
        (Keyword::GenInitial, "gen/initial-value"),
        (Keyword::Height, "height"),
        (Keyword::Inc, "inc"),
        (Keyword::InnerHeight, "inner-height"),
        (Keyword::InnerWidth, "inner-width"),
        (Keyword::Iterations, "iterations"),
        (Keyword::LineWidth, "line-width"),
        (Keyword::LineWidthEnd, "line-width-end"),
        (Keyword::LineWidthMapping, "line-width-mapping"),
        (Keyword::LineWidthStart, "line-width-start"),
        (Keyword::Mapping, "mapping"),
        (Keyword::Max, "max"),
        (Keyword::Min, "min"),
        (Keyword::Num, "num"),
        (Keyword::Overlap, "overlap"),
        (Keyword::Point, "point"),
        (Keyword::Position, "position"),
        (Keyword::Preset, "preset"),
        (Keyword::Radius, "radius"),
        (Keyword::Scalar, "scalar"),
        (Keyword::Seed, "seed"),
        (Keyword::Steps, "steps"),
        (Keyword::StrokeLineWidthEnd, "stroke-line-width-end"),
        (Keyword::StrokeLineWidthStart, "stroke-line-width-start"),
        (Keyword::StrokeNoise, "stroke-noise"),
        (Keyword::StrokeTessellation, "stroke-tessellation"),
        (Keyword::TEnd, "t-end"),
        (Keyword::TStart, "t-start"),
        (Keyword::Tessellation, "tessellation"),
        (Keyword::TransformPosition, "transform-position"),
        (Keyword::To, "to"),
        (Keyword::ToColour, "to-colour"),
        (Keyword::Upto, "upto"),
        (Keyword::Value, "value"),
        (Keyword::Vec1, "vec1"),
        (Keyword::Vec2, "vec2"),
        (Keyword::Vector, "vector"),
        (Keyword::Volatility, "volatility"),
        (Keyword::Width, "width"),
    ];

    for (kw, s) in names.iter() {
        hm.insert(s.to_string(), *kw as i32);
    }

    hm
}

pub fn keyword_to_string_hash() -> HashMap<Keyword, String> {
    let mut hm: HashMap<Keyword, String> = HashMap::new();

    let names = [
        (Keyword::False, "false"),
        (Keyword::True, "true"),
        (Keyword::Plus, "+"),
        (Keyword::Minus, "-"),
        (Keyword::Mult, "*"),
        (Keyword::Divide, "/"),
        (Keyword::Equal, "="),
        (Keyword::Gt, ">"),
        (Keyword::Lt, "<"),
        (Keyword::VectorAppend, "++"),
        (Keyword::Sqrt, "sqrt"),
        (Keyword::Mod, "mod"),
        (Keyword::And, "and"),
        (Keyword::Or, "or"),
        (Keyword::Not, "not"),
        (Keyword::Define, "define"),
        (Keyword::Fn, "fn"),
        (Keyword::If, "if"),
        (Keyword::Each, "each"),
        (Keyword::Loop, "loop"),
        (Keyword::Fence, "fence"),
        (Keyword::OnMatrixStack, "on-matrix-stack"),
        (Keyword::Setq, "setq"),
        (Keyword::AddressOf, "address-of"),
        (Keyword::FnCall, "fn-call"),
        (Keyword::Quote, "quote"),
        (Keyword::HashVars, "#vars"),
        (Keyword::CanvasWidth, "canvas/width"),
        (Keyword::CanvasHeight, "canvas/height"),
        (Keyword::MathTau, "math/TAU"),
        (Keyword::Rgb, "RGB"),
        (Keyword::Hsl, "HSL"),
        (Keyword::Hsluv, "HSLuv"),
        (Keyword::Lab, "LAB"),
        (Keyword::Hsv, "HSV"),
        (Keyword::White, "white"),
        (Keyword::Black, "black"),
        (Keyword::Red, "red"),
        (Keyword::Green, "green"),
        (Keyword::Blue, "blue"),
        (Keyword::Yellow, "yellow"),
        (Keyword::Magenta, "magenta"),
        (Keyword::Cyan, "cyan"),
        (Keyword::Chrome, "chrome"),
        (Keyword::HotlineMiami, "hotline-miami"),
        (Keyword::KnightRider, "knight-rider"),
        (Keyword::Mars, "mars"),
        (Keyword::Rainbow, "rainbow"),
        (Keyword::Robocop, "robocop"),
        (Keyword::Transformers, "transformers"),
        (Keyword::ColProceduralFnPresets, "col/procedural-fn-presets"),
        (Keyword::BrushFlat, "brush-flat"),
        (Keyword::BrushA, "brush-a"),
        (Keyword::BrushB, "brush-b"),
        (Keyword::BrushC, "brush-c"),
        (Keyword::BrushD, "brush-d"),
        (Keyword::BrushE, "brush-e"),
        (Keyword::BrushF, "brush-f"),
        (Keyword::BrushG, "brush-g"),
        (Keyword::Linear, "linear"),
        (Keyword::EaseQuick, "ease/quick"),
        (Keyword::EaseSlowIn, "ease/slow-in"),
        (Keyword::EaseSlowInOut, "ease/slow-in-out"),
        (Keyword::EaseQuadraticIn, "ease/quadratic-in"),
        (Keyword::EaseQuadraticOut, "ease/quadratic-out"),
        (Keyword::EaseQuadraticInOut, "ease/quadratic-in-out"),
        (Keyword::EaseCubicIn, "ease/cubic-in"),
        (Keyword::EaseCubicOut, "ease/cubic-out"),
        (Keyword::EaseCubicInOut, "ease/cubic-in-out"),
        (Keyword::EaseQuarticIn, "ease/quartic-in"),
        (Keyword::EaseQuarticOut, "ease/quartic-out"),
        (Keyword::EaseQuarticInOut, "ease/quartic-in-out"),
        (Keyword::EaseQuinticIn, "ease/quintic-in"),
        (Keyword::EaseQuinticOut, "ease/quintic-out"),
        (Keyword::EaseQuinticInOut, "ease/quintic-in-out"),
        (Keyword::EaseSinIn, "ease/sin-in"),
        (Keyword::EaseSinOut, "ease/sin-out"),
        (Keyword::EaseSinInOut, "ease/sin-in-out"),
        (Keyword::EaseCircularIn, "ease/circular-in"),
        (Keyword::EaseCircularOut, "ease/circular-out"),
        (Keyword::EaseCircularInOut, "ease/circular-in-out"),
        (Keyword::EaseExponentialIn, "ease/exponential-in"),
        (Keyword::EaseExponentialOut, "ease/exponential-out"),
        (Keyword::EaseExponentialInOut, "ease/exponential-in-out"),
        (Keyword::EaseElasticIn, "ease/elastic-in"),
        (Keyword::EaseElasticOut, "ease/elastic-out"),
        (Keyword::EaseElasticInOut, "ease/elastic-in-out"),
        (Keyword::EaseBackIn, "ease/back-in"),
        (Keyword::EaseBackOut, "ease/back-out"),
        (Keyword::EaseBackInOut, "ease/back-in-out"),
        (Keyword::EaseBounceIn, "ease/bounce-in"),
        (Keyword::EaseBounceOut, "ease/bounce-out"),
        (Keyword::EaseBounceInOut, "ease/bounce-in-out"),
        (Keyword::EasePresets, "ease/presets"),
        (Keyword::A, "a"),
        (Keyword::B, "b"),
        (Keyword::C, "c"),
        (Keyword::D, "d"),
        (Keyword::G, "g"),
        (Keyword::H, "h"),
        (Keyword::L, "l"),
        (Keyword::N, "n"),
        (Keyword::R, "r"),
        (Keyword::S, "s"),
        (Keyword::T, "t"),
        (Keyword::V, "v"),
        (Keyword::X, "x"),
        (Keyword::Y, "y"),
        (Keyword::Z, "z"),
        (Keyword::Alpha, "alpha"),
        (Keyword::Amplitude, "amplitude"),
        (Keyword::Angle, "angle"),
        (Keyword::AngleEnd, "angle-end"),
        (Keyword::AngleStart, "angle-start"),
        (Keyword::Brush, "brush"),
        (Keyword::BrushSubtype, "brush-subtype"),
        (Keyword::By, "by"),
        (Keyword::Clamping, "clamping"),
        (Keyword::Colour, "colour"),
        (Keyword::ColourVolatility, "colour-volatility"),
        (Keyword::Colours, "colours"),
        (Keyword::Coords, "coords"),
        (Keyword::Copies, "copies"),
        (Keyword::Copy, "copy"),
        (Keyword::Direction, "direction"),
        (Keyword::Distance, "distance"),
        (Keyword::Format, "format"),
        (Keyword::Frequency, "frequency"),
        (Keyword::From, "from"),
        (Keyword::FromColour, "from-colour"),
        (Keyword::GenInitial, "gen/initial-value"),
        (Keyword::Height, "height"),
        (Keyword::Inc, "inc"),
        (Keyword::InnerHeight, "inner-height"),
        (Keyword::InnerWidth, "inner-width"),
        (Keyword::Iterations, "iterations"),
        (Keyword::LineWidth, "line-width"),
        (Keyword::LineWidthEnd, "line-width-end"),
        (Keyword::LineWidthMapping, "line-width-mapping"),
        (Keyword::LineWidthStart, "line-width-start"),
        (Keyword::Mapping, "mapping"),
        (Keyword::Max, "max"),
        (Keyword::Min, "min"),
        (Keyword::Num, "num"),
        (Keyword::Overlap, "overlap"),
        (Keyword::Point, "point"),
        (Keyword::Position, "position"),
        (Keyword::Preset, "preset"),
        (Keyword::Radius, "radius"),
        (Keyword::Scalar, "scalar"),
        (Keyword::Seed, "seed"),
        (Keyword::Steps, "steps"),
        (Keyword::StrokeLineWidthEnd, "stroke-line-width-end"),
        (Keyword::StrokeLineWidthStart, "stroke-line-width-start"),
        (Keyword::StrokeNoise, "stroke-noise"),
        (Keyword::StrokeTessellation, "stroke-tessellation"),
        (Keyword::TEnd, "t-end"),
        (Keyword::TStart, "t-start"),
        (Keyword::Tessellation, "tessellation"),
        (Keyword::TransformPosition, "transform-position"),
        (Keyword::To, "to"),
        (Keyword::ToColour, "to-colour"),
        (Keyword::Upto, "upto"),
        (Keyword::Value, "value"),
        (Keyword::Vec1, "vec1"),
        (Keyword::Vec2, "vec2"),
        (Keyword::Vector, "vector"),
        (Keyword::Volatility, "volatility"),
        (Keyword::Width, "width"),
    ];

    for (kw, s) in names.iter() {
        hm.insert(*kw, s.to_string());
    }

    hm
}

pub fn keyword_to_string(kw: Keyword) -> String {
    match kw {
        Keyword::False => "false".to_string(),
        Keyword::True => "true".to_string(),
        Keyword::Plus => "+".to_string(),
        Keyword::Minus => "-".to_string(),
        Keyword::Mult => "*".to_string(),
        Keyword::Divide => "/".to_string(),
        Keyword::Equal => "=".to_string(),
        Keyword::Gt => ">".to_string(),
        Keyword::Lt => "<".to_string(),
        Keyword::VectorAppend => "++".to_string(),
        Keyword::Sqrt => "sqrt".to_string(),
        Keyword::Mod => "mod".to_string(),
        Keyword::And => "and".to_string(),
        Keyword::Or => "or".to_string(),
        Keyword::Not => "not".to_string(),
        Keyword::Define => "define".to_string(),
        Keyword::Fn => "fn".to_string(),
        Keyword::If => "if".to_string(),
        Keyword::Each => "each".to_string(),
        Keyword::Loop => "loop".to_string(),
        Keyword::Fence => "fence".to_string(),
        Keyword::OnMatrixStack => "on-matrix-stack".to_string(),
        Keyword::Setq => "setq".to_string(),
        Keyword::AddressOf => "address-of".to_string(),
        Keyword::FnCall => "fn-call".to_string(),
        Keyword::Quote => "quote".to_string(),
        Keyword::HashVars => "#vars".to_string(),
        Keyword::CanvasWidth => "canvas/width".to_string(),
        Keyword::CanvasHeight => "canvas/height".to_string(),
        Keyword::MathTau => "math/TAU".to_string(),
        Keyword::Rgb => "RGB".to_string(),
        Keyword::Hsl => "HSL".to_string(),
        Keyword::Hsluv => "HSLuv".to_string(),
        Keyword::Lab => "LAB".to_string(),
        Keyword::Hsv => "HSV".to_string(),
        Keyword::White => "white".to_string(),
        Keyword::Black => "black".to_string(),
        Keyword::Red => "red".to_string(),
        Keyword::Green => "green".to_string(),
        Keyword::Blue => "blue".to_string(),
        Keyword::Yellow => "yellow".to_string(),
        Keyword::Magenta => "magenta".to_string(),
        Keyword::Cyan => "cyan".to_string(),
        Keyword::Chrome => "chrome".to_string(),
        Keyword::HotlineMiami => "hotline-miami".to_string(),
        Keyword::KnightRider => "knight-rider".to_string(),
        Keyword::Mars => "mars".to_string(),
        Keyword::Rainbow => "rainbow".to_string(),
        Keyword::Robocop => "robocop".to_string(),
        Keyword::Transformers => "transformers".to_string(),
        Keyword::ColProceduralFnPresets => "col/procedural-fn-presets".to_string(),
        Keyword::BrushFlat => "brush-flat".to_string(),
        Keyword::BrushA => "brush-a".to_string(),
        Keyword::BrushB => "brush-b".to_string(),
        Keyword::BrushC => "brush-c".to_string(),
        Keyword::BrushD => "brush-d".to_string(),
        Keyword::BrushE => "brush-e".to_string(),
        Keyword::BrushF => "brush-f".to_string(),
        Keyword::BrushG => "brush-g".to_string(),
        Keyword::Linear => "linear".to_string(),
        Keyword::EaseQuick => "ease/quick".to_string(),
        Keyword::EaseSlowIn => "ease/slow-in".to_string(),
        Keyword::EaseSlowInOut => "ease/slow-in-out".to_string(),
        Keyword::EaseQuadraticIn => "ease/quadratic-in".to_string(),
        Keyword::EaseQuadraticOut => "ease/quadratic-out".to_string(),
        Keyword::EaseQuadraticInOut => "ease/quadratic-in-out".to_string(),
        Keyword::EaseCubicIn => "ease/cubic-in".to_string(),
        Keyword::EaseCubicOut => "ease/cubic-out".to_string(),
        Keyword::EaseCubicInOut => "ease/cubic-in-out".to_string(),
        Keyword::EaseQuarticIn => "ease/quartic-in".to_string(),
        Keyword::EaseQuarticOut => "ease/quartic-out".to_string(),
        Keyword::EaseQuarticInOut => "ease/quartic-in-out".to_string(),
        Keyword::EaseQuinticIn => "ease/quintic-in".to_string(),
        Keyword::EaseQuinticOut => "ease/quintic-out".to_string(),
        Keyword::EaseQuinticInOut => "ease/quintic-in-out".to_string(),
        Keyword::EaseSinIn => "ease/sin-in".to_string(),
        Keyword::EaseSinOut => "ease/sin-out".to_string(),
        Keyword::EaseSinInOut => "ease/sin-in-out".to_string(),
        Keyword::EaseCircularIn => "ease/circular-in".to_string(),
        Keyword::EaseCircularOut => "ease/circular-out".to_string(),
        Keyword::EaseCircularInOut => "ease/circular-in-out".to_string(),
        Keyword::EaseExponentialIn => "ease/exponential-in".to_string(),
        Keyword::EaseExponentialOut => "ease/exponential-out".to_string(),
        Keyword::EaseExponentialInOut => "ease/exponential-in-out".to_string(),
        Keyword::EaseElasticIn => "ease/elastic-in".to_string(),
        Keyword::EaseElasticOut => "ease/elastic-out".to_string(),
        Keyword::EaseElasticInOut => "ease/elastic-in-out".to_string(),
        Keyword::EaseBackIn => "ease/back-in".to_string(),
        Keyword::EaseBackOut => "ease/back-out".to_string(),
        Keyword::EaseBackInOut => "ease/back-in-out".to_string(),
        Keyword::EaseBounceIn => "ease/bounce-in".to_string(),
        Keyword::EaseBounceOut => "ease/bounce-out".to_string(),
        Keyword::EaseBounceInOut => "ease/bounce-in-out".to_string(),
        Keyword::EasePresets => "ease/presets".to_string(),
        Keyword::A => "a".to_string(),
        Keyword::B => "b".to_string(),
        Keyword::C => "c".to_string(),
        Keyword::D => "d".to_string(),
        Keyword::G => "g".to_string(),
        Keyword::H => "h".to_string(),
        Keyword::L => "l".to_string(),
        Keyword::N => "n".to_string(),
        Keyword::R => "r".to_string(),
        Keyword::S => "s".to_string(),
        Keyword::T => "t".to_string(),
        Keyword::V => "v".to_string(),
        Keyword::X => "x".to_string(),
        Keyword::Y => "y".to_string(),
        Keyword::Z => "z".to_string(),
        Keyword::Alpha => "alpha".to_string(),
        Keyword::Amplitude => "amplitude".to_string(),
        Keyword::Angle => "angle".to_string(),
        Keyword::AngleEnd => "angle-end".to_string(),
        Keyword::AngleStart => "angle-start".to_string(),
        Keyword::Brush => "brush".to_string(),
        Keyword::BrushSubtype => "brush-subtype".to_string(),
        Keyword::By => "by".to_string(),
        Keyword::Clamping => "clamping".to_string(),
        Keyword::Colour => "colour".to_string(),
        Keyword::ColourVolatility => "colour-volatility".to_string(),
        Keyword::Colours => "colours".to_string(),
        Keyword::Coords => "coords".to_string(),
        Keyword::Copies => "copies".to_string(),
        Keyword::Copy => "copy".to_string(),
        Keyword::Direction => "direction".to_string(),
        Keyword::Distance => "distance".to_string(),
        Keyword::Format => "format".to_string(),
        Keyword::Frequency => "frequency".to_string(),
        Keyword::From => "from".to_string(),
        Keyword::FromColour => "from-colour".to_string(),
        Keyword::GenInitial => "gen/initial-value".to_string(),
        Keyword::Height => "height".to_string(),
        Keyword::Inc => "inc".to_string(),
        Keyword::InnerHeight => "inner-height".to_string(),
        Keyword::InnerWidth => "inner-width".to_string(),
        Keyword::Iterations => "iterations".to_string(),
        Keyword::LineWidth => "line-width".to_string(),
        Keyword::LineWidthEnd => "line-width-end".to_string(),
        Keyword::LineWidthMapping => "line-width-mapping".to_string(),
        Keyword::LineWidthStart => "line-width-start".to_string(),
        Keyword::Mapping => "mapping".to_string(),
        Keyword::Max => "max".to_string(),
        Keyword::Min => "min".to_string(),
        Keyword::Num => "num".to_string(),
        Keyword::Overlap => "overlap".to_string(),
        Keyword::Point => "point".to_string(),
        Keyword::Position => "position".to_string(),
        Keyword::Preset => "preset".to_string(),
        Keyword::Radius => "radius".to_string(),
        Keyword::Scalar => "scalar".to_string(),
        Keyword::Seed => "seed".to_string(),
        Keyword::Steps => "steps".to_string(),
        Keyword::StrokeLineWidthEnd => "stroke-line-width-end".to_string(),
        Keyword::StrokeLineWidthStart => "stroke-line-width-start".to_string(),
        Keyword::StrokeNoise => "stroke-noise".to_string(),
        Keyword::StrokeTessellation => "stroke-tessellation".to_string(),
        Keyword::TEnd => "t-end".to_string(),
        Keyword::TStart => "t-start".to_string(),
        Keyword::Tessellation => "tessellation".to_string(),
        Keyword::TransformPosition => "transform-position".to_string(),
        Keyword::To => "to".to_string(),
        Keyword::ToColour => "to-colour".to_string(),
        Keyword::Upto => "upto".to_string(),
        Keyword::Value => "value".to_string(),
        Keyword::Vec1 => "vec1".to_string(),
        Keyword::Vec2 => "vec2".to_string(),
        Keyword::Vector => "vector".to_string(),
        Keyword::Volatility => "volatility".to_string(),
        Keyword::Width => "width".to_string(),
    }
}
