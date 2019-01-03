// Copyright (C) 2019 Inderjit Gill

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

use crate::bind::*;
use crate::compiler::Program;
use crate::error::Result;
use crate::keywords::Keyword;
use crate::vm::{Var, Vm};

use std::collections::HashMap;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Native {
    NativeStart = Keyword::KeywordEnd as isize,

    // misc
    //
    DebugPrint,
    Nth,
    VectorLength,

    // shapes
    //
    Line,
    Rect,
    Circle,
    CircleSlice,
    Poly,
    Bezier,
    BezierBulging,
    StrokedBezier,
    StrokedBezierRect,

    // transforms
    //
    Translate,
    Rotate,
    Scale,

    // colour
    //
    ColConvert,
    ColRGB, // start of colour constructors
    ColHSL,
    ColHSLuv,
    ColHSV,
    ColLAB, // end of colour constructors
    ColComplementary,
    ColSplitComplementary,
    ColAnalagous,
    ColTriad,
    ColDarken,
    ColLighten,
    ColSetAlpha,
    ColGetAlpha,
    ColSetR,
    ColGetR,
    ColSetG,
    ColGetG,
    ColSetB,
    ColGetB,
    ColSetH,
    ColGetH,
    ColSetS,
    ColGetS,
    ColSetL,
    ColGetL,
    ColSetA,
    ColGetA,
    ColSetV,
    ColGetV,
    ColBuildProcedural,
    ColBuildBezier,
    ColValue,

    // math
    //
    MathDistance,
    MathNormal,
    MathClamp,
    MathRadiansDegrees,
    MathCos,
    MathSin,

    // prng
    //
    PrngBuild,
    PrngValues,
    PrngValue,
    PrngPerlin,

    // interp
    //
    InterpBuild,
    InterpValue,
    InterpCos,
    InterpSin,
    InterpBezier,
    InterpBezierTangent,
    InterpRay,
    InterpLine,
    InterpCircle,

    // path
    //
    PathLinear,
    PathCircle,
    PathSpline,
    PathBezier,

    // repeat
    //
    RepeatSymmetryVertical,
    RepeatSymmetryHorizontal,
    RepeatSymmetry4,
    RepeatSymmetry8,
    RepeatRotate,
    RepeatRotateMirror,

    // focal
    //
    FocalBuildPoint,
    FocalBuildVLine,
    FocalBuildHLine,
    FocalValue,

    // gen
    //
    GenStrayInt,
    GenStray,
    GenStray2D,
    GenStray3D,
    GenStray4D,
    GenInt,
    GenScalar,
    Gen2D,
    GenSelect,
    GenCol,

    NativeEnd,
}

impl fmt::Display for Native {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Native::DebugPrint => write!(f, "debug/print"),
            Native::Nth => write!(f, "nth"),
            Native::VectorLength => write!(f, "vector/length"),
            Native::Line => write!(f, "line"),
            Native::Rect => write!(f, "rect"),
            Native::Circle => write!(f, "circle"),
            Native::CircleSlice => write!(f, "circle-slice"),
            Native::Poly => write!(f, "poly"),
            Native::Bezier => write!(f, "bezier"),
            Native::BezierBulging => write!(f, "bezier-bulging"),
            Native::StrokedBezier => write!(f, "stroked-bezier"),
            Native::StrokedBezierRect => write!(f, "stroked-bezier-rect"),
            Native::Translate => write!(f, "translate"),
            Native::Rotate => write!(f, "rotate"),
            Native::Scale => write!(f, "scale"),
            Native::ColConvert => write!(f, "col/convert"),
            Native::ColRGB => write!(f, "col/rgb"),
            Native::ColHSL => write!(f, "col/hsl"),
            Native::ColHSLuv => write!(f, "col/hsluv"),
            Native::ColHSV => write!(f, "col/hsv"),
            Native::ColLAB => write!(f, "col/lab"),
            Native::ColComplementary => write!(f, "col/complementary"),
            Native::ColSplitComplementary => write!(f, "col/split-complementary"),
            Native::ColAnalagous => write!(f, "col/analagous"),
            Native::ColTriad => write!(f, "col/triad"),
            Native::ColDarken => write!(f, "col/darken"),
            Native::ColLighten => write!(f, "col/lighten"),
            Native::ColSetAlpha => write!(f, "col/set-alpha"),
            Native::ColGetAlpha => write!(f, "col/get-alpha"),
            Native::ColSetR => write!(f, "col/set-r"),
            Native::ColGetR => write!(f, "col/get-r"),
            Native::ColSetG => write!(f, "col/set-g"),
            Native::ColGetG => write!(f, "col/get-g"),
            Native::ColSetB => write!(f, "col/set-b"),
            Native::ColGetB => write!(f, "col/get-b"),
            Native::ColSetH => write!(f, "col/set-h"),
            Native::ColGetH => write!(f, "col/get-h"),
            Native::ColSetS => write!(f, "col/set-s"),
            Native::ColGetS => write!(f, "col/get-s"),
            Native::ColSetL => write!(f, "col/set-l"),
            Native::ColGetL => write!(f, "col/get-l"),
            Native::ColSetA => write!(f, "col/set-a"),
            Native::ColGetA => write!(f, "col/get-a"),
            Native::ColSetV => write!(f, "col/set-v"),
            Native::ColGetV => write!(f, "col/get-v"),
            Native::ColBuildProcedural => write!(f, "col/build-procedural"),
            Native::ColBuildBezier => write!(f, "col/build-bezier"),
            Native::ColValue => write!(f, "col/value"),
            Native::MathDistance => write!(f, "math/distance"),
            Native::MathNormal => write!(f, "math/normal"),
            Native::MathClamp => write!(f, "math/clamp"),
            Native::MathRadiansDegrees => write!(f, "math/radians->degrees"),
            Native::MathCos => write!(f, "math/cos"),
            Native::MathSin => write!(f, "math/sin"),
            Native::PrngBuild => write!(f, "prng/build"),
            Native::PrngValues => write!(f, "prng/values"),
            Native::PrngValue => write!(f, "prng/value"),
            Native::PrngPerlin => write!(f, "prng/perlin"),
            Native::InterpBuild => write!(f, "interp/build"),
            Native::InterpValue => write!(f, "interp/value"),
            Native::InterpCos => write!(f, "interp/cos"),
            Native::InterpSin => write!(f, "interp/sin"),
            Native::InterpBezier => write!(f, "interp/bezier"),
            Native::InterpBezierTangent => write!(f, "interp/bezier-tangent"),
            Native::InterpRay => write!(f, "interp/ray"),
            Native::InterpLine => write!(f, "interp/line"),
            Native::InterpCircle => write!(f, "interp/circle"),
            Native::PathLinear => write!(f, "path/linear"),
            Native::PathCircle => write!(f, "path/circle"),
            Native::PathSpline => write!(f, "path/spline"),
            Native::PathBezier => write!(f, "path/bezier"),
            Native::RepeatSymmetryVertical => write!(f, "repeat/symmetry-vertical"),
            Native::RepeatSymmetryHorizontal => write!(f, "repeat/symmetry-horizontal"),
            Native::RepeatSymmetry4 => write!(f, "repeat/symmetry-4"),
            Native::RepeatSymmetry8 => write!(f, "repeat/symmetry-8"),
            Native::RepeatRotate => write!(f, "repeat/rotate"),
            Native::RepeatRotateMirror => write!(f, "repeat/rotate-mirrored"),
            Native::FocalBuildPoint => write!(f, "focal/build-point"),
            Native::FocalBuildVLine => write!(f, "focal/build-vline"),
            Native::FocalBuildHLine => write!(f, "focal/build-hline"),
            Native::FocalValue => write!(f, "focal/value"),
            Native::GenStrayInt => write!(f, "gen/stray-int"),
            Native::GenStray => write!(f, "gen/stray"),
            Native::GenStray2D => write!(f, "gen/stray-2d"),
            Native::GenStray3D => write!(f, "gen/stray-3d"),
            Native::GenStray4D => write!(f, "gen/stray-4d"),
            Native::GenInt => write!(f, "gen/int"),
            Native::GenScalar => write!(f, "gen/scalar"),
            Native::Gen2D => write!(f, "gen/2d"),
            Native::GenSelect => write!(f, "gen/select"),
            Native::GenCol => write!(f, "gen/col"),
            _ => write!(f, "Unknown: {}", *self as i32),
        }
    }
}

pub fn string_to_native(s: &str) -> Option<Native> {
    match s {
        "debug/print" => Some(Native::DebugPrint),
        "nth" => Some(Native::Nth),
        "vector/length" => Some(Native::VectorLength),
        "line" => Some(Native::Line),
        "rect" => Some(Native::Rect),
        "circle" => Some(Native::Circle),
        "circle-slice" => Some(Native::CircleSlice),
        "poly" => Some(Native::Poly),
        "bezier" => Some(Native::Bezier),
        "bezier-bulging" => Some(Native::BezierBulging),
        "stroked-bezier" => Some(Native::StrokedBezier),
        "stroked-bezier-rect" => Some(Native::StrokedBezierRect),
        "translate" => Some(Native::Translate),
        "rotate" => Some(Native::Rotate),
        "scale" => Some(Native::Scale),
        "col/convert" => Some(Native::ColConvert),
        "col/rgb" => Some(Native::ColRGB), // start of colour constructors
        "col/hsl" => Some(Native::ColHSL),
        "col/hsluv" => Some(Native::ColHSLuv),
        "col/hsv" => Some(Native::ColHSV),
        "col/lab" => Some(Native::ColLAB), // end of colour constructors
        "col/complementary" => Some(Native::ColComplementary),
        "col/split-complementary" => Some(Native::ColSplitComplementary),
        "col/analagous" => Some(Native::ColAnalagous),
        "col/triad" => Some(Native::ColTriad),
        "col/darken" => Some(Native::ColDarken),
        "col/lighten" => Some(Native::ColLighten),
        "col/set-alpha" => Some(Native::ColSetAlpha),
        "col/get-alpha" => Some(Native::ColGetAlpha),
        "col/set-r" => Some(Native::ColSetR),
        "col/get-r" => Some(Native::ColGetR),
        "col/set-g" => Some(Native::ColSetG),
        "col/get-g" => Some(Native::ColGetG),
        "col/set-b" => Some(Native::ColSetB),
        "col/get-b" => Some(Native::ColGetB),
        "col/set-h" => Some(Native::ColSetH),
        "col/get-h" => Some(Native::ColGetH),
        "col/set-s" => Some(Native::ColSetS),
        "col/get-s" => Some(Native::ColGetS),
        "col/set-l" => Some(Native::ColSetL),
        "col/get-l" => Some(Native::ColGetL),
        "col/set-a" => Some(Native::ColSetA),
        "col/get-a" => Some(Native::ColGetA),
        "col/set-v" => Some(Native::ColSetV),
        "col/get-v" => Some(Native::ColGetV),
        "col/build-procedural" => Some(Native::ColBuildProcedural),
        "col/build-bezier" => Some(Native::ColBuildBezier),
        "col/value" => Some(Native::ColValue),
        "math/distance" => Some(Native::MathDistance),
        "math/normal" => Some(Native::MathNormal),
        "math/clamp" => Some(Native::MathClamp),
        "math/radians->degrees" => Some(Native::MathRadiansDegrees),
        "math/cos" => Some(Native::MathCos),
        "math/sin" => Some(Native::MathSin),
        "prng/build" => Some(Native::PrngBuild),
        "prng/values" => Some(Native::PrngValues),
        "prng/value" => Some(Native::PrngValue),
        "prng/perlin" => Some(Native::PrngPerlin),
        "interp/build" => Some(Native::InterpBuild),
        "interp/value" => Some(Native::InterpValue),
        "interp/cos" => Some(Native::InterpCos),
        "interp/sin" => Some(Native::InterpSin),
        "interp/bezier" => Some(Native::InterpBezier),
        "interp/bezier-tangent" => Some(Native::InterpBezierTangent),
        "interp/ray" => Some(Native::InterpRay),
        "interp/line" => Some(Native::InterpLine),
        "interp/circle" => Some(Native::InterpCircle),
        "path/linear" => Some(Native::PathLinear),
        "path/circle" => Some(Native::PathCircle),
        "path/spline" => Some(Native::PathSpline),
        "path/bezier" => Some(Native::PathBezier),
        "repeat/symmetry-vertical" => Some(Native::RepeatSymmetryVertical),
        "repeat/symmetry-horizontal" => Some(Native::RepeatSymmetryHorizontal),
        "repeat/symmetry-4" => Some(Native::RepeatSymmetry4),
        "repeat/symmetry-8" => Some(Native::RepeatSymmetry8),
        "repeat/rotate" => Some(Native::RepeatRotate),
        "repeat/rotate-mirrored" => Some(Native::RepeatRotateMirror),
        "focal/build-point" => Some(Native::FocalBuildPoint),
        "focal/build-vline" => Some(Native::FocalBuildVLine),
        "focal/build-hline" => Some(Native::FocalBuildHLine),
        "focal/value" => Some(Native::FocalValue),
        "gen/stray-int" => Some(Native::GenStrayInt),
        "gen/stray" => Some(Native::GenStray),
        "gen/stray-2d" => Some(Native::GenStray2D),
        "gen/stray-3d" => Some(Native::GenStray3D),
        "gen/stray-4d" => Some(Native::GenStray4D),
        "gen/int" => Some(Native::GenInt),
        "gen/scalar" => Some(Native::GenScalar),
        "gen/2d" => Some(Native::Gen2D),
        "gen/select" => Some(Native::GenSelect),
        "gen/col" => Some(Native::GenCol),
        _ => None,
    }
}

pub fn build_native_fn_hash() -> HashMap<Native, fn(&mut Vm, &Program, usize) -> Result<Var>> {
    let mut native_fns: HashMap<Native, fn(&mut Vm, &Program, usize) -> Result<Var>> = HashMap::new();
    native_fns.insert(Native::VectorLength, bind_vector_length);

    native_fns
}
