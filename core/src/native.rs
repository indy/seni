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

/*

known functions


 */

use crate::builtin::Builtin;
use crate::colour::ColourFormat;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::packable::{Mule, Packable};
use crate::vm::{Var, Vm};

use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString, EnumIter)]
pub enum Native {
    #[strum(serialize = "UnreachableNativeStart")]
    NativeStart = Builtin::BuiltinEnd as isize,

    // // misc
    // //
    // #[strum(serialize = "debug/print")]
    // DebugPrint,
    // #[strum(serialize = "nth")]
    // Nth,
    // #[strum(serialize = "vector/length")]
    // VectorLength,
    // #[strum(serialize = "probe")]
    // Probe,

    // shapes
    //
    // #[strum(serialize = "line")]
    // Line,
    #[strum(serialize = "rect")]
    Rect,
    // #[strum(serialize = "circle")]
    // Circle,
    // #[strum(serialize = "circle-slice")]
    // CircleSlice,
    // #[strum(serialize = "poly")]
    // Poly,
    // #[strum(serialize = "quadratic")]
    // Quadratic,
    // #[strum(serialize = "bezier")]
    // Bezier,
    // #[strum(serialize = "bezier-bulging")]
    // BezierBulging,
    // #[strum(serialize = "stroked-bezier")]
    // StrokedBezier,
    // #[strum(serialize = "stroked-bezier-rect")]
    // StrokedBezierRect,

    // // transforms
    // //
    // #[strum(serialize = "translate")]
    // Translate,
    // #[strum(serialize = "rotate")]
    // Rotate,
    // #[strum(serialize = "scale")]
    // Scale,

    // // colour
    // //
    // #[strum(serialize = "col/convert")]
    // ColConvert,
    // #[strum(serialize = "__colour_constructor_start")]
    // ColConstructorStart_, // Special Enums required by the compiler to recognise colour constructors
    // #[strum(serialize = "col/rgb")]
    // ColRGB,
    // #[strum(serialize = "col/hsl")]
    // ColHSL,
    // #[strum(serialize = "col/hsluv")]
    // ColHSLuv,
    // #[strum(serialize = "col/hsv")]
    // ColHSV,
    // #[strum(serialize = "col/lab")]
    // ColLAB,
    // #[strum(serialize = "__colour_constructor_end")]
    // ColConstructorEnd_, // Special Enums required by the compiler to recognise colour constructors
    // #[strum(serialize = "col/complementary")]
    // ColComplementary,
    // #[strum(serialize = "col/split-complementary")]
    // ColSplitComplementary,
    // #[strum(serialize = "col/analagous")]
    // ColAnalagous,
    // #[strum(serialize = "col/triad")]
    // ColTriad,
    // #[strum(serialize = "col/darken")]
    // ColDarken,
    // #[strum(serialize = "col/lighten")]
    // ColLighten,
    // #[strum(serialize = "col/set-alpha")]
    // ColSetAlpha,
    // #[strum(serialize = "col/get-alpha")]
    // ColGetAlpha,
    // #[strum(serialize = "col/set-r")]
    // ColSetR,
    // #[strum(serialize = "col/get-r")]
    // ColGetR,
    // #[strum(serialize = "col/set-g")]
    // ColSetG,
    // #[strum(serialize = "col/get-g")]
    // ColGetG,
    // #[strum(serialize = "col/set-b")]
    // ColSetB,
    // #[strum(serialize = "col/get-b")]
    // ColGetB,
    // #[strum(serialize = "col/set-h")]
    // ColSetH,
    // #[strum(serialize = "col/get-h")]
    // ColGetH,
    // #[strum(serialize = "col/set-s")]
    // ColSetS,
    // #[strum(serialize = "col/get-s")]
    // ColGetS,
    // #[strum(serialize = "col/set-l")]
    // ColSetL,
    // #[strum(serialize = "col/get-l")]
    // ColGetL,
    // #[strum(serialize = "col/set-a")]
    // ColSetA,
    // #[strum(serialize = "col/get-a")]
    // ColGetA,
    // #[strum(serialize = "col/set-v")]
    // ColSetV,
    // #[strum(serialize = "col/get-v")]
    // ColGetV,
    // #[strum(serialize = "col/build-procedural")]
    // ColBuildProcedural,
    // #[strum(serialize = "col/build-bezier")]
    // ColBuildBezier,
    // #[strum(serialize = "col/value")]
    // ColValue,

    // // math
    // //
    // #[strum(serialize = "math/distance")]
    // MathDistance,
    // #[strum(serialize = "math/normal")]
    // MathNormal,
    // #[strum(serialize = "math/clamp")]
    // MathClamp,
    // #[strum(serialize = "math/radians->degrees")]
    // MathRadiansDegrees,
    // #[strum(serialize = "math/cos")]
    // MathCos,
    // #[strum(serialize = "math/sin")]
    // MathSin,

    // // prng
    // //
    // #[strum(serialize = "prng/build")]
    // PrngBuild,
    // #[strum(serialize = "prng/values")]
    // PrngValues,
    // #[strum(serialize = "prng/value")]
    // PrngValue,
    // #[strum(serialize = "prng/perlin")]
    // PrngPerlin,

    // // interp
    // //
    // #[strum(serialize = "interp/build")]
    // InterpBuild,
    // #[strum(serialize = "interp/value")]
    // InterpValue,
    // #[strum(serialize = "interp/cos")]
    // InterpCos,
    // #[strum(serialize = "interp/sin")]
    // InterpSin,
    // #[strum(serialize = "interp/bezier")]
    // InterpBezier,
    // #[strum(serialize = "interp/bezier-tangent")]
    // InterpBezierTangent,
    // #[strum(serialize = "interp/ray")]
    // InterpRay,
    // #[strum(serialize = "interp/line")]
    // InterpLine,
    // #[strum(serialize = "interp/circle")]
    // InterpCircle,

    // // path
    // //
    // #[strum(serialize = "path/linear")]
    // PathLinear,
    // #[strum(serialize = "path/circle")]
    // PathCircle,
    // #[strum(serialize = "path/spline")]
    // PathSpline,
    // #[strum(serialize = "path/bezier")]
    // PathBezier,

    // // repeat
    // //
    // #[strum(serialize = "repeat/symmetry-vertical")]
    // RepeatSymmetryVertical,
    // #[strum(serialize = "repeat/symmetry-horizontal")]
    // RepeatSymmetryHorizontal,
    // #[strum(serialize = "repeat/symmetry-4")]
    // RepeatSymmetry4,
    // #[strum(serialize = "repeat/symmetry-8")]
    // RepeatSymmetry8,
    // #[strum(serialize = "repeat/rotate")]
    // RepeatRotate,
    // #[strum(serialize = "repeat/rotate-mirrored")]
    // RepeatRotateMirrored,

    // // focal
    // //
    // #[strum(serialize = "focal/build-point")]
    // FocalBuildPoint,
    // #[strum(serialize = "focal/build-vline")]
    // FocalBuildVLine,
    // #[strum(serialize = "focal/build-hline")]
    // FocalBuildHLine,
    // #[strum(serialize = "focal/value")]
    // FocalValue,

    // // gen
    // //
    // #[strum(serialize = "gen/stray-int")]
    // GenStrayInt,
    // #[strum(serialize = "gen/stray")]
    // GenStray,
    // #[strum(serialize = "gen/stray-2d")]
    // GenStray2D,
    // #[strum(serialize = "gen/stray-3d")]
    // GenStray3D,
    // #[strum(serialize = "gen/stray-4d")]
    // GenStray4D,
    // #[strum(serialize = "gen/int")]
    // GenInt,
    // #[strum(serialize = "gen/scalar")]
    // GenScalar,
    // #[strum(serialize = "gen/2d")]
    // Gen2D,
    // #[strum(serialize = "gen/select")]
    // GenSelect,
    // #[strum(serialize = "gen/col")]
    // GenCol,
    #[strum(serialize = "UnreachableNativeEnd")]
    NativeEnd,
}

impl Packable for Native {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_label(cursor, &self.to_string());

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<Native>()?;

        Ok((res, &cursor[ns..]))
    }
}

pub fn i32_to_native_hash() -> HashMap<i32, Native> {
    let mut hm: HashMap<i32, Native> = HashMap::new();

    for n in Native::iter() {
        hm.insert(n as i32, n);
    }

    hm
}

// return a tuple
// .0 == input arguments as a vector of (name, default value) pairs
// .1 == how the native function affects the vm's stack in terms of opcode offset
//
pub fn parameter_info(native: &Native) -> Result<(Vec<(Keyword, Var)>, i32)> {
    match native {
        Native::Rect => rect_parameter_info(),
        _ => Err(Error::Native("parameter_info".to_string())),
    }
}

pub fn execute_native(vm: &mut Vm, native: &Native) -> Result<()> {
    match native {
        Native::Rect => rect_execute(vm),
        _ => Err(Error::Native("execute_native".to_string())),
    }
}

fn rect_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Width, Var::Float(4.0)),
            (Keyword::Height, Var::Float(10.0)),
            (Keyword::Position, Var::V2D(10.0, 10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
        ],
        // stack offset
        0,
    ))
}

fn rect_execute(vm: &mut Vm) -> Result<()> {
    let width = vm.stack_peek_f32(1)?;
    let height = vm.stack_peek_f32(2)?;
    let position = vm.stack_peek_v2d(3)?;
    let col = vm.stack_peek_col(4)?;

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {
        vm.render_rect(position, width, height, &rgb)
    } else {
        Err(Error::Native("rect".to_string()))
    }
}
