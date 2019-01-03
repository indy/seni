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

use crate::compiler::Program;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::mathutil;
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
    let mut native_fns: HashMap<Native, fn(&mut Vm, &Program, usize) -> Result<Var>> =
        HashMap::new();

    // --------------------------------------------------
    // misc
    // --------------------------------------------------
    // BIND("debug/print", bind_debug_print);
    native_fns.insert(Native::Nth, bind_nth);
    native_fns.insert(Native::VectorLength, bind_vector_length);
    // map (todo)

    // --------------------------------------------------
    // shapes
    // --------------------------------------------------
    // BIND("line", bind_line);
    // BIND("rect", bind_rect);
    // BIND("circle", bind_circle);
    // BIND("circle-slice", bind_circle_slice);
    // BIND("poly", bind_poly);
    // BIND("bezier", bind_bezier);
    // BIND("bezier-bulging", bind_bezier_bulging);
    // BIND("stroked-bezier", bind_stroked_bezier);
    // BIND("stroked-bezier-rect", bind_stroked_bezier_rect);

    // --------------------------------------------------
    // transforms
    // --------------------------------------------------
    // BIND("translate", bind_translate);
    // BIND("rotate", bind_rotate);
    // BIND("scale", bind_scale);

    // --------------------------------------------------
    // colour
    // --------------------------------------------------
    // BIND("col/convert", bind_col_convert);
    // start of colour constructors
    // g_colour_constructor_start = word_lut->native_count;
    // BIND("col/rgb", bind_col_rgb);
    // BIND("col/hsl", bind_col_hsl);
    // BIND("col/hsluv", bind_col_hsluv);
    // BIND("col/hsv", bind_col_hsv);
    // BIND("col/lab", bind_col_lab);
    // g_colour_constructor_end = word_lut->native_count;
    // end of colour constructors
    // BIND("col/complementary", bind_col_complementary);
    // BIND("col/split-complementary", bind_col_split_complementary);
    // BIND("col/analagous", bind_col_analagous);
    // BIND("col/triad", bind_col_triad);
    // BIND("col/darken", bind_col_darken);
    // BIND("col/lighten", bind_col_lighten);
    // BIND("col/set-alpha", bind_col_set_alpha);
    // BIND("col/get-alpha", bind_col_get_alpha);
    // BIND("col/set-r", bind_col_set_r);
    // BIND("col/get-r", bind_col_get_r);
    // BIND("col/set-g", bind_col_set_g);
    // BIND("col/get-g", bind_col_get_g);
    // BIND("col/set-b", bind_col_set_b);
    // BIND("col/get-b", bind_col_get_b);
    // BIND("col/set-h", bind_col_set_h);
    // BIND("col/get-h", bind_col_get_h);
    // BIND("col/set-s", bind_col_set_s);
    // BIND("col/get-s", bind_col_get_s);
    // BIND("col/set-l", bind_col_set_l);
    // BIND("col/get-l", bind_col_get_l);
    // BIND("col/set-a", bind_col_set_a);
    // BIND("col/get-a", bind_col_get_a);
    // BIND("col/set-v", bind_col_set_v);
    // BIND("col/get-v", bind_col_get_v);
    // BIND("col/build-procedural", bind_col_build_procedural);
    // BIND("col/build-bezier", bind_col_build_bezier);
    // BIND("col/value", bind_col_value);

    // --------------------------------------------------
    // math
    // --------------------------------------------------
    native_fns.insert(Native::MathDistance, bind_math_distance);
    native_fns.insert(Native::MathNormal, bind_math_normal);
    native_fns.insert(Native::MathClamp, bind_math_clamp);
    native_fns.insert(Native::MathRadiansDegrees, bind_math_radians_to_degrees);
    native_fns.insert(Native::MathCos, bind_math_cos);
    native_fns.insert(Native::MathSin, bind_math_sin);

    // --------------------------------------------------
    // prng
    // --------------------------------------------------
    // BIND("prng/build", bind_prng_build);
    // BIND("prng/values", bind_prng_values);
    // BIND("prng/value", bind_prng_value);
    // BIND("prng/perlin", bind_prng_perlin);

    // --------------------------------------------------
    // interp
    // --------------------------------------------------
    // BIND("interp/build", bind_interp_build);
    // BIND("interp/value", bind_interp_value);
    // BIND("interp/cos", bind_interp_cos);
    // BIND("interp/sin", bind_interp_sin);
    // BIND("interp/bezier", bind_interp_bezier);
    // BIND("interp/bezier-tangent", bind_interp_bezier_tangent);
    // BIND("interp/ray", bind_interp_ray);
    // BIND("interp/line", bind_interp_line);
    // BIND("interp/circle", bind_interp_circle);

    // --------------------------------------------------
    // path
    // --------------------------------------------------
    // BIND("path/linear", bind_path_linear);
    // BIND("path/circle", bind_path_circle);
    // BIND("path/spline", bind_path_spline);
    // BIND("path/bezier", bind_path_bezier);

    // --------------------------------------------------
    // repeat
    // --------------------------------------------------
    // BIND("repeat/symmetry-vertical", bind_repeat_symmetry_vertical);
    // BIND("repeat/symmetry-horizontal", bind_repeat_symmetry_horizontal);
    // BIND("repeat/symmetry-4", bind_repeat_symmetry_4);
    // BIND("repeat/symmetry-8", bind_repeat_symmetry_8);
    // BIND("repeat/rotate", bind_repeat_rotate);
    // BIND("repeat/rotate-mirrored", bind_repeat_rotate_mirrored);

    // --------------------------------------------------
    // focal
    // --------------------------------------------------
    // BIND("focal/build-point", bind_focal_build_point);
    // BIND("focal/build-vline", bind_focal_build_vline);
    // BIND("focal/build-hline", bind_focal_build_hline);
    // BIND("focal/value", bind_focal_value);

    // --------------------------------------------------
    // gen
    // --------------------------------------------------
    // BIND("gen/stray-int", bind_gen_stray_int);
    // BIND("gen/stray", bind_gen_stray);
    // BIND("gen/stray-2d", bind_gen_stray_2d);
    // BIND("gen/stray-3d", bind_gen_stray_3d);
    // BIND("gen/stray-4d", bind_gen_stray_4d);
    // BIND("gen/int", bind_gen_int);
    // BIND("gen/scalar", bind_gen_scalar);
    // BIND("gen/2d", bind_gen_2d);
    // BIND("gen/select", bind_gen_select); // broken?
    // BIND("gen/col", bind_gen_col);

    native_fns
}

pub fn bind_nth(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<&Var> = None;
    let mut n: Option<usize> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer + 0];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            if *iname == Keyword::From as i32 {
                from = Some(value);
            }

            if *iname == Keyword::N as i32 {
                if let Var::Float(f) = value {
                    n = Some(*f as usize);
                }
            }
        }
    }

    if let Some(from) = from {
        if let Var::Vector(vs) = from {
            if let Some(nth) = vs.get(n.unwrap_or(0)) {
                return Ok(nth.clone());
            } else {
                return Err(Error::Bind("bind_nth: n out of range".to_string()));
            }
        } else if let Var::V2D(a, b) = from {
            match n.unwrap_or(0) {
                0 => return Ok(Var::Float(*a)),
                1 => return Ok(Var::Float(*b)),
                _ => return Err(Error::Bind("bind_nth: n out of range".to_string())),
            }
        }
    }

    return Err(Error::Bind(
        "bind_nth requires vector argument in 'from'".to_string(),
    ));
}

pub fn bind_vector_length(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vector: Option<&Var> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer + 0];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            if *iname == Keyword::Vector as i32 {
                vector = Some(value);
            }
        }
    }

    if let Some(v) = vector {
        if let Var::Vector(vs) = v {
            let len = vs.len();
            return Ok(Var::Int(len as i32));
        }
    }

    return Err(Error::Bind(
        "bind_vector_length requires vector argument".to_string(),
    ));
}

pub fn bind_math_distance(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vec1: Option<&Var> = None;
    let mut vec2: Option<&Var> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer + 0];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            if *iname == Keyword::Vec1 as i32 {
                vec1 = Some(value);
            }
            if *iname == Keyword::Vec2 as i32 {
                vec2 = Some(value);
            }
        }
    }

    if let Some(vec1_) = vec1 {
        if let Var::V2D(x1, y1) = vec1_ {
            if let Some(vec2_) = vec2 {
                if let Var::V2D(x2, y2) = vec2_ {
                    let distance = mathutil::distance_v2(*x1, *y1, *x2, *y2);
                    return Ok(Var::Float(distance));
                }
            }
        }
    }

    return Err(Error::Bind("bind error".to_string()));
}

pub fn bind_math_normal(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vec1: Option<&Var> = None;
    let mut vec2: Option<&Var> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer + 0];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            if *iname == Keyword::Vec1 as i32 {
                vec1 = Some(value);
            }
            if *iname == Keyword::Vec2 as i32 {
                vec2 = Some(value);
            }
        }
    }

    if let Some(vec1_) = vec1 {
        if let Var::V2D(x1, y1) = vec1_ {
            if let Some(vec2_) = vec2 {
                if let Var::V2D(x2, y2) = vec2_ {
                    let norm = mathutil::normal(*x1, *y1, *x2, *y2);
                    return Ok(Var::V2D(norm.0, norm.1));
                }
            }
        }
    }

    return Err(Error::Bind("bind error".to_string()));
}

pub fn bind_math_clamp(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    // todo: try and move functions like this into ones that initially
    // create and return a function that takes a single argument.
    // e.g.
    // (define my-clamp (math/clamp-fn min: 0.0 max: 42.0))
    // (my-clamp val: 22)
    //
    // then optimize for single argument functions as these will be much faster to
    // parse
    //

    let mut value: Option<f32> = None;
    let mut min: Option<f32> = None;
    let mut max: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label_ = &vm.stack[args_pointer + 0];
        let value_ = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname_) = label_ {
            if *iname_ == Keyword::Value as i32 {
                if let Var::Float(f) = value_ {
                    value = Some(*f);
                }
            }
            if *iname_ == Keyword::Min as i32 {
                if let Var::Float(f) = value_ {
                    min = Some(*f);
                }
            }
            if *iname_ == Keyword::Max as i32 {
                if let Var::Float(f) = value_ {
                    max = Some(*f);
                }
            }
        }
    }

    let res = mathutil::clamp(value.unwrap_or(0.0), min.unwrap_or(0.0), max.unwrap_or(0.0));

    return Ok(Var::Float(res));
}

pub fn bind_math_radians_to_degrees(
    vm: &mut Vm,
    _program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut angle: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label_ = &vm.stack[args_pointer + 0];
        let value_ = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname_) = label_ {
            if *iname_ == Keyword::Angle as i32 {
                if let Var::Float(f) = value_ {
                    angle = Some(*f);
                }
            }
        }
    }

    let res = mathutil::rad_to_deg(angle.unwrap_or(0.0));

    return Ok(Var::Float(res));
}

pub fn bind_math_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut angle: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label_ = &vm.stack[args_pointer + 0];
        let value_ = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname_) = label_ {
            if *iname_ == Keyword::Angle as i32 {
                if let Var::Float(f) = value_ {
                    angle = Some(*f);
                }
            }
        }
    }

    let res = angle.unwrap_or(0.0).cos();

    return Ok(Var::Float(res));
}

pub fn bind_math_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut angle: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label_ = &vm.stack[args_pointer + 0];
        let value_ = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname_) = label_ {
            if *iname_ == Keyword::Angle as i32 {
                if let Var::Float(f) = value_ {
                    angle = Some(*f);
                }
            }
        }
    }

    let res = angle.unwrap_or(0.0).sin();

    return Ok(Var::Float(res));
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_bind_nth() {
        is_float("(define v [1 2 3 4]) (nth from: v n: 0)", 1.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 1)", 2.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 2)", 3.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 3)", 4.0);

        is_float("(define v [9 8]) (nth from: v n: 0)", 9.0);
        is_float("(define v [9 8]) (nth from: v n: 1)", 8.0);
    }

    #[test]
    fn test_bind_vector_length() {
        is_int("(define v []) (++ v 100) (vector/length vector: v)", 1);
        is_int("(define v [1]) (++ v 100) (vector/length vector: v)", 2);
        is_int("(define v [1 2]) (++ v 100) (vector/length vector: v)", 3);
        is_int("(define v [1 2 3]) (++ v 100) (vector/length vector: v)", 4);
        is_int(
            "(define v [1 2 3 4]) (++ v 100) (vector/length vector: v)",
            5,
        );
        is_int(
            "(define v []) (++ v 4) (++ v 3) (++ v 2) (++ v 1) (++ v 0) (vector/length vector: v)",
            5,
        );
        is_int(
            "(define v [1 2]) (++ v 98) (++ v 99) (++ v 100) (vector/length vector: v)",
            5,
        );
    }

    #[test]
    fn test_bind_math() {
        is_float("(math/clamp value: 3 min: 2 max: 5)", 3.0);
        is_float("(math/clamp value: 1 min: 2 max: 5)", 2.0);
        is_float("(math/clamp value: 8 min: 2 max: 5)", 5.0);

        is_float("(math/radians->degrees angle: 0.3)", 17.188734);

        is_float("(math/cos angle: 0.7)", 0.7648422);
        is_float("(math/sin angle: 0.9)", 0.7833269);
    }

}
