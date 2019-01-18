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

use crate::colour::{Colour, ColourFormat};
use crate::compiler::Program;
use crate::ease::*;
use crate::error::{Error, Result};
use crate::interp::*;
use crate::keywords::Keyword;
use crate::mathutil;
use crate::path::*;
use crate::repeat::*;
use crate::uvmapper::BrushType;
use crate::vm::{Var, Vm};

use std::collections::HashMap;

use strum_macros::EnumString;

pub type NativeCallback = fn(&mut Vm, &Program, usize) -> Result<Var>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumString)]
pub enum Native {
    #[strum(serialize = "UnreachableNativeStart")]
    NativeStart = Keyword::KeywordEnd as isize,

    // misc
    //
    #[strum(serialize = "debug/print")]
    DebugPrint,
    #[strum(serialize = "nth")]
    Nth,
    #[strum(serialize = "vector/length")]
    VectorLength,
    #[strum(serialize = "probe")]
    Probe,

    // shapes
    //
    #[strum(serialize = "line")]
    Line,
    #[strum(serialize = "rect")]
    Rect,
    #[strum(serialize = "circle")]
    Circle,
    #[strum(serialize = "circle-slice")]
    CircleSlice,
    #[strum(serialize = "poly")]
    Poly,
    #[strum(serialize = "quadratic")]
    Quadratic,
    #[strum(serialize = "bezier")]
    Bezier,
    #[strum(serialize = "bezier-bulging")]
    BezierBulging,
    #[strum(serialize = "stroked-bezier")]
    StrokedBezier,
    #[strum(serialize = "stroked-bezier-rect")]
    StrokedBezierRect,

    // transforms
    //
    #[strum(serialize = "translate")]
    Translate,
    #[strum(serialize = "rotate")]
    Rotate,
    #[strum(serialize = "scale")]
    Scale,

    // colour
    //
    #[strum(serialize = "col/convert")]
    ColConvert,
    #[strum(serialize = "__colour_constructor_start")]
    ColConstructorStart_, // Special Enums required by the compiler to recognise colour constructors
    #[strum(serialize = "col/rgb")]
    ColRGB,
    #[strum(serialize = "col/hsl")]
    ColHSL,
    #[strum(serialize = "col/hsluv")]
    ColHSLuv,
    #[strum(serialize = "col/hsv")]
    ColHSV,
    #[strum(serialize = "col/lab")]
    ColLAB,
    #[strum(serialize = "__colour_constructor_end")]
    ColConstructorEnd_, // Special Enums required by the compiler to recognise colour constructors
    #[strum(serialize = "col/complementary")]
    ColComplementary,
    #[strum(serialize = "col/split-complementary")]
    ColSplitComplementary,
    #[strum(serialize = "col/analagous")]
    ColAnalagous,
    #[strum(serialize = "col/triad")]
    ColTriad,
    #[strum(serialize = "col/darken")]
    ColDarken,
    #[strum(serialize = "col/lighten")]
    ColLighten,
    #[strum(serialize = "col/set-alpha")]
    ColSetAlpha,
    #[strum(serialize = "col/get-alpha")]
    ColGetAlpha,
    #[strum(serialize = "col/set-r")]
    ColSetR,
    #[strum(serialize = "col/get-r")]
    ColGetR,
    #[strum(serialize = "col/set-g")]
    ColSetG,
    #[strum(serialize = "col/get-g")]
    ColGetG,
    #[strum(serialize = "col/set-b")]
    ColSetB,
    #[strum(serialize = "col/get-b")]
    ColGetB,
    #[strum(serialize = "col/set-h")]
    ColSetH,
    #[strum(serialize = "col/get-h")]
    ColGetH,
    #[strum(serialize = "col/set-s")]
    ColSetS,
    #[strum(serialize = "col/get-s")]
    ColGetS,
    #[strum(serialize = "col/set-l")]
    ColSetL,
    #[strum(serialize = "col/get-l")]
    ColGetL,
    #[strum(serialize = "col/set-a")]
    ColSetA,
    #[strum(serialize = "col/get-a")]
    ColGetA,
    #[strum(serialize = "col/set-v")]
    ColSetV,
    #[strum(serialize = "col/get-v")]
    ColGetV,
    #[strum(serialize = "col/build-procedural")]
    ColBuildProcedural,
    #[strum(serialize = "col/build-bezier")]
    ColBuildBezier,
    #[strum(serialize = "col/value")]
    ColValue,

    // math
    //
    #[strum(serialize = "math/distance")]
    MathDistance,
    #[strum(serialize = "math/normal")]
    MathNormal,
    #[strum(serialize = "math/clamp")]
    MathClamp,
    #[strum(serialize = "math/radians->degrees")]
    MathRadiansDegrees,
    #[strum(serialize = "math/cos")]
    MathCos,
    #[strum(serialize = "math/sin")]
    MathSin,

    // prng
    //
    #[strum(serialize = "prng/build")]
    PrngBuild,
    #[strum(serialize = "prng/values")]
    PrngValues,
    #[strum(serialize = "prng/value")]
    PrngValue,
    #[strum(serialize = "prng/perlin")]
    PrngPerlin,

    // interp
    //
    #[strum(serialize = "interp/build")]
    InterpBuild,
    #[strum(serialize = "interp/value")]
    InterpValue,
    #[strum(serialize = "interp/cos")]
    InterpCos,
    #[strum(serialize = "interp/sin")]
    InterpSin,
    #[strum(serialize = "interp/bezier")]
    InterpBezier,
    #[strum(serialize = "interp/bezier-tangent")]
    InterpBezierTangent,
    #[strum(serialize = "interp/ray")]
    InterpRay,
    #[strum(serialize = "interp/line")]
    InterpLine,
    #[strum(serialize = "interp/circle")]
    InterpCircle,

    // path
    //
    #[strum(serialize = "path/linear")]
    PathLinear,
    #[strum(serialize = "path/circle")]
    PathCircle,
    #[strum(serialize = "path/spline")]
    PathSpline,
    #[strum(serialize = "path/bezier")]
    PathBezier,

    // repeat
    //
    #[strum(serialize = "repeat/symmetry-vertical")]
    RepeatSymmetryVertical,
    #[strum(serialize = "repeat/symmetry-horizontal")]
    RepeatSymmetryHorizontal,
    #[strum(serialize = "repeat/symmetry-4")]
    RepeatSymmetry4,
    #[strum(serialize = "repeat/symmetry-8")]
    RepeatSymmetry8,
    #[strum(serialize = "repeat/rotate")]
    RepeatRotate,
    #[strum(serialize = "repeat/rotate-mirrored")]
    RepeatRotateMirrored,

    // focal
    //
    #[strum(serialize = "focal/build-point")]
    FocalBuildPoint,
    #[strum(serialize = "focal/build-vline")]
    FocalBuildVLine,
    #[strum(serialize = "focal/build-hline")]
    FocalBuildHLine,
    #[strum(serialize = "focal/value")]
    FocalValue,

    // gen
    //
    #[strum(serialize = "gen/stray-int")]
    GenStrayInt,
    #[strum(serialize = "gen/stray")]
    GenStray,
    #[strum(serialize = "gen/stray-2d")]
    GenStray2D,
    #[strum(serialize = "gen/stray-3d")]
    GenStray3D,
    #[strum(serialize = "gen/stray-4d")]
    GenStray4D,
    #[strum(serialize = "gen/int")]
    GenInt,
    #[strum(serialize = "gen/scalar")]
    GenScalar,
    #[strum(serialize = "gen/2d")]
    Gen2D,
    #[strum(serialize = "gen/select")]
    GenSelect,
    #[strum(serialize = "gen/col")]
    GenCol,

    #[strum(serialize = "UnreachableNativeEnd")]
    NativeEnd,
}

pub fn build_native_fn_hash() -> HashMap<Native, NativeCallback> {
    let mut native_fns: HashMap<Native, NativeCallback> = HashMap::new();

    // --------------------------------------------------
    // misc
    // --------------------------------------------------
    // BIND("debug/print", bind_debug_print);
    native_fns.insert(Native::Nth, bind_nth);
    native_fns.insert(Native::VectorLength, bind_vector_length);
    native_fns.insert(Native::Probe, bind_probe);
    // map (todo)

    // --------------------------------------------------
    // shapes
    // --------------------------------------------------
    native_fns.insert(Native::Line, bind_line);
    native_fns.insert(Native::Rect, bind_rect);
    native_fns.insert(Native::Circle, bind_circle);
    native_fns.insert(Native::CircleSlice, bind_circle_slice);
    native_fns.insert(Native::Poly, bind_poly);
    native_fns.insert(Native::Quadratic, bind_quadratic);
    native_fns.insert(Native::Bezier, bind_bezier);
    native_fns.insert(Native::BezierBulging, bind_bezier_bulging);
    // BIND("stroked-bezier", bind_stroked_bezier);
    // BIND("stroked-bezier-rect", bind_stroked_bezier_rect);

    // --------------------------------------------------
    // transforms
    // --------------------------------------------------
    native_fns.insert(Native::Translate, bind_translate);
    native_fns.insert(Native::Rotate, bind_rotate);
    native_fns.insert(Native::Scale, bind_scale);

    // --------------------------------------------------
    // colour
    // --------------------------------------------------
    // BIND("col/convert", bind_col_convert);
    native_fns.insert(Native::ColRGB, bind_col_rgb);
    native_fns.insert(Native::ColHSL, bind_col_hsl);
    native_fns.insert(Native::ColHSLuv, bind_col_hsluv);
    native_fns.insert(Native::ColHSV, bind_col_hsv);
    native_fns.insert(Native::ColLAB, bind_col_lab);
    // BIND("col/complementary", bind_col_complementary);
    // BIND("col/split-complementary", bind_col_split_complementary);
    // BIND("col/analagous", bind_col_analagous);
    // BIND("col/triad", bind_col_triad);
    // BIND("col/darken", bind_col_darken);
    // BIND("col/lighten", bind_col_lighten);
    native_fns.insert(Native::ColSetAlpha, bind_col_set_alpha);
    native_fns.insert(Native::ColGetAlpha, bind_col_get_alpha);
    native_fns.insert(Native::ColSetR, bind_col_set_r);
    native_fns.insert(Native::ColGetR, bind_col_get_r);
    native_fns.insert(Native::ColSetG, bind_col_set_g);
    native_fns.insert(Native::ColGetG, bind_col_get_g);
    native_fns.insert(Native::ColSetB, bind_col_set_b);
    native_fns.insert(Native::ColGetB, bind_col_get_b);
    native_fns.insert(Native::ColSetH, bind_col_set_h);
    native_fns.insert(Native::ColGetH, bind_col_get_h);
    native_fns.insert(Native::ColSetS, bind_col_set_s);
    native_fns.insert(Native::ColGetS, bind_col_get_s);
    native_fns.insert(Native::ColSetL, bind_col_set_l);
    native_fns.insert(Native::ColGetL, bind_col_get_l);
    native_fns.insert(Native::ColSetA, bind_col_set_a);
    native_fns.insert(Native::ColGetA, bind_col_get_a);
    native_fns.insert(Native::ColSetV, bind_col_set_v);
    native_fns.insert(Native::ColGetV, bind_col_get_v);
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
    native_fns.insert(Native::InterpCos, bind_interp_cos);
    native_fns.insert(Native::InterpSin, bind_interp_sin);
    native_fns.insert(Native::InterpBezier, bind_interp_bezier);
    native_fns.insert(Native::InterpBezierTangent, bind_interp_bezier_tangent);
    native_fns.insert(Native::InterpRay, bind_interp_ray);
    native_fns.insert(Native::InterpLine, bind_interp_line);
    native_fns.insert(Native::InterpCircle, bind_interp_circle);

    // --------------------------------------------------
    // path
    // --------------------------------------------------
    native_fns.insert(Native::PathLinear, bind_path_linear);
    native_fns.insert(Native::PathCircle, bind_path_circle);
    native_fns.insert(Native::PathSpline, bind_path_spline);
    native_fns.insert(Native::PathBezier, bind_path_bezier);

    // --------------------------------------------------
    // repeat
    // --------------------------------------------------
    native_fns.insert(
        Native::RepeatSymmetryVertical,
        bind_repeat_symmetry_vertical,
    );
    native_fns.insert(
        Native::RepeatSymmetryHorizontal,
        bind_repeat_symmetry_horizontal,
    );
    native_fns.insert(Native::RepeatSymmetry4, bind_repeat_symmetry_4);
    native_fns.insert(Native::RepeatSymmetry8, bind_repeat_symmetry_8);
    native_fns.insert(Native::RepeatRotate, bind_repeat_rotate);
    native_fns.insert(Native::RepeatRotateMirrored, bind_repeat_mirrored);

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

fn read_i32(iname: i32, value: &Var, keyword: Keyword) -> Option<i32> {
    if iname == keyword as i32 {
        if let Var::Int(i) = value {
            return Some(*i);
        }
    }
    None
}

fn read_float(iname: i32, value: &Var, keyword: Keyword) -> Option<f32> {
    if iname == keyword as i32 {
        if let Var::Float(f) = value {
            return Some(*f);
        }
    }
    None
}

fn read_float_as_usize(iname: i32, value: &Var, keyword: Keyword) -> Option<usize> {
    if iname == keyword as i32 {
        if let Var::Float(f) = value {
            return Some(*f as usize);
        }
    }
    None
}

fn read_v2d(iname: i32, value: &Var, keyword: Keyword) -> Option<(f32, f32)> {
    if iname == keyword as i32 {
        if let Var::V2D(x, y) = value {
            return Some((*x, *y));
        }
    }
    None
}

fn read_vector(iname: i32, value: &Var, keyword: Keyword) -> Option<&Vec<Var>> {
    if iname == keyword as i32 {
        if let Var::Vector(vecs) = value {
            return Some(vecs);
        }
    }
    None
}

fn read_kw(iname: i32, value: &Var, keyword: Keyword) -> Option<Keyword> {
    if iname == keyword as i32 {
        if let Var::Keyword(kw) = value {
            return Some(*kw);
        }
    }
    None
}

fn read_col(
    iname: i32,
    value: &Var,
    keyword: Keyword,
) -> Option<(ColourFormat, f32, f32, f32, f32)> {
    if iname == keyword as i32 {
        if let Var::Colour(fmt, e0, e1, e2, e3) = value {
            return Some((*fmt, *e0, *e1, *e2, *e3));
        }
    }
    None
}

fn read_brush(iname: i32, value: &Var, keyword: Keyword) -> Option<BrushType> {
    if iname == keyword as i32 {
        if let Var::Keyword(n) = value {
            let brush = match *n {
                Keyword::BrushFlat => BrushType::Flat,
                Keyword::BrushA => BrushType::A,
                Keyword::BrushB => BrushType::B,
                Keyword::BrushC => BrushType::C,
                Keyword::BrushD => BrushType::D,
                Keyword::BrushE => BrushType::E,
                Keyword::BrushF => BrushType::F,
                Keyword::BrushG => BrushType::G,
                _ => BrushType::Flat,
            };
            return Some(brush);
        }
    }
    None
}

macro_rules! read_i32 {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_i32(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_float {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_float(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_float_as_usize {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_float_as_usize(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_v2d {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_v2d(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_vector {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_vector(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_kw {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_kw(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_col {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_col(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_brush {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_brush(*$in, $v, $kw).or($i);
    };
}

fn array_f32_6_from_vec(float_pairs: &[Var]) -> [f32; 6] {
    let mut res = [0.0; 6];

    for i in 0..3 {
        if let Var::V2D(x, y) = float_pairs[i] {
            res[i * 2] = x;
            res[(i * 2) + 1] = y;
        }
    }

    res
}

fn array_f32_8_from_vec(float_pairs: &[Var]) -> [f32; 8] {
    let mut res = [0.0; 8];

    for i in 0..4 {
        if let Var::V2D(x, y) = float_pairs[i] {
            res[i * 2] = x;
            res[(i * 2) + 1] = y;
        }
    }

    res
}

fn rgb_tuples_from_colour_tuples(
    col: &(ColourFormat, f32, f32, f32, f32),
) -> Result<(f32, f32, f32, f32)> {
    let (fmt, e0, e1, e2, e3) = col;

    if *fmt == ColourFormat::Rgb {
        Ok((*e0, *e1, *e2, *e3))
    } else {
        let colour = Colour::build_colour_from_elements(*fmt, &(*e0, *e1, *e2, *e3));
        colour.to_rgba32_tuple()
    }
}

pub fn bind_nth(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<&Var> = None;
    let mut n: Option<usize> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
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

    Err(Error::Bind(
        "bind_nth requires vector argument in 'from'".to_string(),
    ))
}

pub fn bind_vector_length(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vector: Option<&Var> = None;

    let mut args_pointer = vm.sp - (num_args * 2);

    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
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

    Err(Error::Bind(
        "bind_vector_length requires vector argument".to_string(),
    ))
}

pub fn bind_probe(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut scalar: Option<f32> = None;
    let mut vector: Option<(f32, f32)> = None;
    let mut worldspace: Option<(f32, f32)> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(scalar, Keyword::Scalar, iname, value);
            read_v2d!(vector, Keyword::Vector, iname, value);
            read_v2d!(worldspace, Keyword::WorldSpace, iname, value);
        }
    }

    if let Some(f) = scalar {
        vm.debug_str_append(&format!("{}", f));
    }

    if let Some((x, y)) = vector {
        vm.debug_str_append(&format!("({},{})", x, y));
    }

    if let Some((x, y)) = worldspace {
        if let Some(matrix) = vm.matrix_stack.peek() {
            let (nx, ny) = matrix.transform_vec2(x, y);
            vm.debug_str_append(&format!("({},{})", nx, ny));
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut from: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut to: Option<(f32, f32)> = Some((900.0, 900.0));
    let mut from_colour: Option<(ColourFormat, f32, f32, f32, f32)> = None;
    let mut to_colour: Option<(ColourFormat, f32, f32, f32, f32)> = None;
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    // let mut s: String = "".to_string();

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(width, Keyword::Width, iname, value);
            read_v2d!(from, Keyword::From, iname, value);
            read_v2d!(to, Keyword::To, iname, value);
            read_col!(from_colour, Keyword::FromColour, iname, value);
            read_col!(to_colour, Keyword::ToColour, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_line matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    let from_col = if let Some((fmt, fr, fg, fb, fa)) = from_colour {
        (fmt, fr, fg, fb, fa)
    } else {
        colour.unwrap()
    };

    let to_col = if let Some((fmt, tr, tg, tb, ta)) = to_colour {
        (fmt, tr, tg, tb, ta)
    } else {
        colour.unwrap()
    };

    if let Ok(from_rgb_tuples) = rgb_tuples_from_colour_tuples(&from_col) {
        if let Ok(to_rgb_tuples) = rgb_tuples_from_colour_tuples(&to_col) {
            vm.geometry.render_line(
                matrix,
                from.unwrap(),
                to.unwrap(),
                width.unwrap(),
                from_rgb_tuples,
                to_rgb_tuples,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
    // Ok(Var::Debug(format!("s: {}", s).to_string()))
}

pub fn bind_rect(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(width, Keyword::Width, iname, value);
            read_float!(height, Keyword::Height, iname, value);
            read_v2d!(position, Keyword::Position, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
        vm.geometry.render_rect(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            rgb_tuples,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut tessellation: Option<f32> = Some(10.0);
    let mut radius: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(width, Keyword::Width, iname, value);
            read_float!(height, Keyword::Height, iname, value);
            read_v2d!(position, Keyword::Position, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_float!(radius, Keyword::Radius, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    // if the radius has been defined then it overrides the width and height parameters
    if let Some(r) = radius {
        width = Some(r);
        height = Some(r);
    }

    if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
        vm.geometry.render_circle(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            rgb_tuples,
            tessellation.unwrap() as usize,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_circle_slice(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut tessellation: Option<f32> = Some(10.0);
    let mut radius: Option<f32> = None;
    let mut angle_start: Option<f32> = Some(0.0);
    let mut angle_end: Option<f32> = Some(10.0);
    let mut inner_width: Option<f32> = Some(1.0);
    let mut inner_height: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(width, Keyword::Width, iname, value);
            read_float!(height, Keyword::Height, iname, value);
            read_v2d!(position, Keyword::Position, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_float!(radius, Keyword::Radius, iname, value);
            read_float!(angle_start, Keyword::AngleStart, iname, value);
            read_float!(angle_end, Keyword::AngleEnd, iname, value);
            read_float!(inner_width, Keyword::InnerWidth, iname, value);
            read_float!(inner_height, Keyword::InnerHeight, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    // if the radius has been defined then it overrides the width and height parameters
    if let Some(r) = radius {
        width = Some(r);
        height = Some(r);
    }

    if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
        vm.geometry.render_circle_slice(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            rgb_tuples,
            tessellation.unwrap() as usize,
            angle_start.unwrap(),
            angle_end.unwrap(),
            inner_width.unwrap(),
            inner_height.unwrap(),
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_poly(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut coords: Option<&Vec<Var>> = None;
    let mut colours: Option<&Vec<Var>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(coords, Keyword::Coords, iname, value);
            read_vector!(colours, Keyword::Colours, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    if let Some(coords_) = coords {
        if let Some(colours_) = colours {
            vm.geometry.render_poly(matrix, coords_, colours_, uvm)?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_quadratic(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = None;
    let mut line_width_start: Option<f32> = Some(4.0);
    let mut line_width_end: Option<f32> = Some(4.0);
    let mut line_width_mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(line_width, Keyword::LineWidth, iname, value);
            read_float!(line_width_start, Keyword::LineWidthStart, iname, value);
            read_float!(line_width_end, Keyword::LineWidthEnd, iname, value);
            read_kw!(line_width_mapping, Keyword::LineWidthMapping, iname, value);
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    // if the line has been defined then it overrides the line_width_start, line_width_end parameters
    let width_start = if let Some(lw) = line_width {
        lw
    } else {
        line_width_start.unwrap()
    };
    let width_end = if let Some(lw) = line_width {
        lw
    } else {
        line_width_end.unwrap()
    };

    let co = array_f32_6_from_vec(coords.unwrap());

    let maybe_mapping = easing_from_keyword(line_width_mapping.unwrap());
    if let Some(mapping) = maybe_mapping {
        if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
            vm.geometry.render_quadratic(
                matrix,
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                rgb_tuples,
                tessellation.unwrap() as usize,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_bezier(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = None;
    let mut line_width_start: Option<f32> = Some(4.0);
    let mut line_width_end: Option<f32> = Some(4.0);
    let mut line_width_mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(line_width, Keyword::LineWidth, iname, value);
            read_float!(line_width_start, Keyword::LineWidthStart, iname, value);
            read_float!(line_width_end, Keyword::LineWidthEnd, iname, value);
            read_kw!(line_width_mapping, Keyword::LineWidthMapping, iname, value);
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    // if the line has been defined then it overrides the line_width_start, line_width_end parameters
    let width_start = if let Some(lw) = line_width {
        lw
    } else {
        line_width_start.unwrap()
    };
    let width_end = if let Some(lw) = line_width {
        lw
    } else {
        line_width_end.unwrap()
    };

    let co = array_f32_8_from_vec(coords.unwrap());

    let maybe_mapping = easing_from_keyword(line_width_mapping.unwrap());
    if let Some(mapping) = maybe_mapping {
        if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
            vm.geometry.render_bezier(
                matrix,
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                rgb_tuples,
                tessellation.unwrap() as usize,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_bezier_bulging(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = Some(4.0);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(line_width, Keyword::LineWidth, iname, value);
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bind_bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    let co = array_f32_8_from_vec(coords.unwrap());

    if let Ok(rgb_tuples) = rgb_tuples_from_colour_tuples(&colour.unwrap()) {
        vm.geometry.render_bezier_bulging(
            matrix,
            &co,
            line_width.unwrap(),
            t_start.unwrap(),
            t_end.unwrap(),
            rgb_tuples,
            tessellation.unwrap() as usize,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_translate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vector: Option<(f32, f32)> = Some((0.0, 0.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(vector, Keyword::Vector, iname, value);
        }
    }

    if let Some((x, y)) = vector {
        vm.matrix_stack.translate(x, y);
    }

    Ok(Var::Bool(true))
}

pub fn bind_rotate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut angle: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(angle, Keyword::Angle, iname, value);
        }
    }

    if let Some(a) = angle {
        vm.matrix_stack.rotate(mathutil::deg_to_rad(a));
    }

    Ok(Var::Bool(true))
}

pub fn bind_scale(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vector: Option<(f32, f32)> = Some((1.0, 1.0));
    let mut scalar: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(vector, Keyword::Vector, iname, value);
            read_float!(scalar, Keyword::Scalar, iname, value);
        }
    }

    if let Some(s) = scalar {
        vm.matrix_stack.scale(s, s);
    } else if let Some((sx, sy)) = vector {
        vm.matrix_stack.scale(sx, sy);
    }

    Ok(Var::Bool(true))
}

pub fn bind_col_rgb(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    // (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)

    let mut r: Option<f32> = Some(0.0);
    let mut g: Option<f32> = Some(0.0);
    let mut b: Option<f32> = Some(0.0);
    let mut alpha: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(r, Keyword::R, iname, value);
            read_float!(g, Keyword::G, iname, value);
            read_float!(b, Keyword::B, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    Ok(Var::Colour(
        ColourFormat::Rgb,
        r.unwrap(),
        g.unwrap(),
        b.unwrap(),
        alpha.unwrap(),
    ))
}

pub fn bind_col_hsl(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut h: Option<f32> = Some(0.0);
    let mut s: Option<f32> = Some(0.0);
    let mut l: Option<f32> = Some(0.0);
    let mut alpha: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(h, Keyword::H, iname, value);
            read_float!(s, Keyword::S, iname, value);
            read_float!(l, Keyword::L, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    Ok(Var::Colour(
        ColourFormat::Hsl,
        h.unwrap(),
        s.unwrap(),
        l.unwrap(),
        alpha.unwrap(),
    ))
}

pub fn bind_col_hsluv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut h: Option<f32> = Some(0.0);
    let mut s: Option<f32> = Some(0.0);
    let mut l: Option<f32> = Some(0.0);
    let mut alpha: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(h, Keyword::H, iname, value);
            read_float!(s, Keyword::S, iname, value);
            read_float!(l, Keyword::L, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    Ok(Var::Colour(
        ColourFormat::Hsluv,
        h.unwrap(),
        s.unwrap(),
        l.unwrap(),
        alpha.unwrap(),
    ))
}

pub fn bind_col_hsv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut h: Option<f32> = Some(0.0);
    let mut s: Option<f32> = Some(0.0);
    let mut v: Option<f32> = Some(0.0);
    let mut alpha: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(h, Keyword::H, iname, value);
            read_float!(s, Keyword::S, iname, value);
            read_float!(v, Keyword::V, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    Ok(Var::Colour(
        ColourFormat::Hsv,
        h.unwrap(),
        s.unwrap(),
        v.unwrap(),
        alpha.unwrap(),
    ))
}

pub fn bind_col_lab(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut l: Option<f32> = Some(0.0);
    let mut a: Option<f32> = Some(0.0);
    let mut b: Option<f32> = Some(0.0);
    let mut alpha: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(l, Keyword::L, iname, value);
            read_float!(a, Keyword::A, iname, value);
            read_float!(b, Keyword::B, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    Ok(Var::Colour(
        ColourFormat::Lab,
        l.unwrap(),
        a.unwrap(),
        b.unwrap(),
        alpha.unwrap(),
    ))
}

pub fn bind_col_set_elem(
    idx: usize,
    vm: &mut Vm,
    _program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));
    let mut val: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
            read_float!(val, Keyword::Value, iname, value);
        }
    }

    if let Some((fmt, e0, e1, e2, e3)) = colour {
        match idx {
            0 => Ok(Var::Colour(fmt, val.unwrap(), e1, e2, e3)),
            1 => Ok(Var::Colour(fmt, e0, val.unwrap(), e2, e3)),
            2 => Ok(Var::Colour(fmt, e0, e1, val.unwrap(), e3)),
            3 => Ok(Var::Colour(fmt, e0, e1, e2, val.unwrap())),
            _ => Err(Error::Bind(
                "bind_col_set_elem::idx out of range".to_string(),
            )),
        }
    } else {
        Err(Error::Bind("unreachable".to_string()))
    }
}

pub fn bind_col_get_elem(
    idx: usize,
    vm: &mut Vm,
    _program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut colour: Option<(ColourFormat, f32, f32, f32, f32)> =
        Some((ColourFormat::Rgb, 0.0, 0.0, 0.0, 1.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    let (_fmt, e0, e1, e2, e3) = colour.unwrap();
    match idx {
        0 => Ok(Var::Float(e0)),
        1 => Ok(Var::Float(e1)),
        2 => Ok(Var::Float(e2)),
        3 => Ok(Var::Float(e3)),
        _ => Err(Error::Bind(
            "bind_col_get_elem::idx out of range".to_string(),
        )),
    }
}

pub fn bind_col_set_alpha(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(3, vm, program, num_args)
}

pub fn bind_col_get_alpha(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(3, vm, program, num_args)
}

pub fn bind_col_set_r(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(0, vm, program, num_args)
}

pub fn bind_col_get_r(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(0, vm, program, num_args)
}

pub fn bind_col_set_g(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(1, vm, program, num_args)
}

pub fn bind_col_get_g(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(1, vm, program, num_args)
}

pub fn bind_col_set_b(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(2, vm, program, num_args)
}

pub fn bind_col_get_b(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(2, vm, program, num_args)
}

pub fn bind_col_set_h(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(0, vm, program, num_args)
}

pub fn bind_col_get_h(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(0, vm, program, num_args)
}

pub fn bind_col_set_s(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(1, vm, program, num_args)
}

pub fn bind_col_get_s(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(1, vm, program, num_args)
}

pub fn bind_col_set_l(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(2, vm, program, num_args)
}

pub fn bind_col_get_l(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(2, vm, program, num_args)
}

pub fn bind_col_set_a(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(1, vm, program, num_args)
}

pub fn bind_col_get_a(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(1, vm, program, num_args)
}

pub fn bind_col_set_v(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_set_elem(2, vm, program, num_args)
}

pub fn bind_col_get_v(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    bind_col_get_elem(2, vm, program, num_args)
}

pub fn bind_math_distance(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vec1: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut vec2: Option<(f32, f32)> = Some((0.0, 0.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(vec1, Keyword::Vec1, iname, value);
            read_v2d!(vec2, Keyword::Vec2, iname, value);
        }
    }

    let v1 = vec1.unwrap();
    let v2 = vec2.unwrap();

    let distance = mathutil::distance_v2(v1.0, v1.1, v2.0, v2.1);
    Ok(Var::Float(distance))
}

pub fn bind_math_normal(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut vec1: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut vec2: Option<(f32, f32)> = Some((0.0, 0.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(vec1, Keyword::Vec1, iname, value);
            read_v2d!(vec2, Keyword::Vec2, iname, value);
        }
    }

    let v1 = vec1.unwrap();
    let v2 = vec2.unwrap();

    let norm = mathutil::normal(v1.0, v1.1, v2.0, v2.1);
    Ok(Var::V2D(norm.0, norm.1))
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

    let mut val: Option<f32> = Some(0.0);
    let mut min: Option<f32> = Some(0.0);
    let mut max: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(val, Keyword::Value, iname, value);
            read_float!(min, Keyword::Min, iname, value);
            read_float!(max, Keyword::Max, iname, value);
        }
    }

    let res = mathutil::clamp(val.unwrap(), min.unwrap(), max.unwrap());
    Ok(Var::Float(res))
}

pub fn bind_math_radians_to_degrees(
    vm: &mut Vm,
    _program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut angle: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(angle, Keyword::Angle, iname, value);
        }
    }

    let res = mathutil::rad_to_deg(angle.unwrap());
    Ok(Var::Float(res))
}

pub fn bind_math_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut angle: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(angle, Keyword::Angle, iname, value);
        }
    }

    let res = angle.unwrap().cos();
    Ok(Var::Float(res))
}

pub fn bind_math_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut angle: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(angle, Keyword::Angle, iname, value);
        }
    }

    let res = angle.unwrap().sin();
    Ok(Var::Float(res))
}

pub fn bind_interp_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut amplitude: Option<f32> = Some(1.0);
    let mut frequency: Option<f32> = Some(1.0);
    let mut t: Option<f32> = Some(1.0); // t goes from 0 to TAU

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(amplitude, Keyword::Amplitude, iname, value);
            read_float!(frequency, Keyword::Frequency, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let res = interp_cos(amplitude.unwrap(), frequency.unwrap(), t.unwrap());

    Ok(Var::Float(res))
}

pub fn bind_interp_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut amplitude: Option<f32> = Some(1.0);
    let mut frequency: Option<f32> = Some(1.0);
    let mut t: Option<f32> = Some(1.0); // t goes from 0 to TAU

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(amplitude, Keyword::Amplitude, iname, value);
            read_float!(frequency, Keyword::Frequency, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let res = interp_sin(amplitude.unwrap(), frequency.unwrap(), t.unwrap());

    Ok(Var::Float(res))
}

pub fn bind_interp_bezier(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut coords: Option<&Vec<Var>> = None;
    let mut t: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let co = array_f32_8_from_vec(coords.unwrap());
    let (x, y) = interp_bezier(&co, t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn bind_interp_bezier_tangent(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut coords: Option<&Vec<Var>> = None;
    let mut t: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let co = array_f32_8_from_vec(coords.unwrap());
    let (x, y) = interp_bezier_tangent(&co, t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn bind_interp_ray(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut point: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut direction: Option<(f32, f32)> = Some((1000.0, 1000.0));
    let mut t: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(point, Keyword::Point, iname, value);
            read_v2d!(direction, Keyword::Direction, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let (x, y) = interp_ray(point.unwrap(), direction.unwrap(), t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn bind_interp_line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut to: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut clamping: Option<Keyword> = Some(Keyword::False);
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut t: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(from, Keyword::From, iname, value);
            read_v2d!(to, Keyword::To, iname, value);
            read_kw!(clamping, Keyword::Clamping, iname, value);
            read_kw!(mapping, Keyword::Mapping, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let maybe_mapping = easing_from_keyword(mapping.unwrap());
    if let Some(mapping) = maybe_mapping {
        let from_ = from.unwrap();
        let to_ = to.unwrap();

        let clamping_ = clamping.unwrap() == Keyword::True;
        let t_ = t.unwrap();

        let x = interp_scalar(from_.0, to_.0, mapping, clamping_, t_);
        let y = interp_scalar(from_.1, to_.1, mapping, clamping_, t_);

        return Ok(Var::V2D(x, y));
    }

    Err(Error::Bind("bind_interp_line".to_string()))
}

pub fn bind_interp_circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut position: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut radius: Option<f32> = Some(1.0);
    let mut t: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(position, Keyword::Position, iname, value);
            read_float!(radius, Keyword::Radius, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    let (x, y) = interp_circle(position.unwrap(), radius.unwrap(), t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn bind_path_linear(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    // (path/linear fn: foo steps: 10 from: [0 0] to: [100 100])

    let mut from_vec: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut to_vec: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut steps: Option<f32> = Some(10.0);
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut fun: Option<i32> = Some(-1);
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(from_vec, Keyword::From, iname, value);
            read_v2d!(to_vec, Keyword::To, iname, value);
            read_float!(steps, Keyword::Steps, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_i32!(fun, Keyword::Fn, iname, value);
            read_kw!(mapping, Keyword::Mapping, iname, value);
        }
    }

    let fr = from_vec.unwrap();
    let to = to_vec.unwrap();
    let maybe_mapping = easing_from_keyword(mapping.unwrap());

    if let Some(mapping) = maybe_mapping {
        path_linear(
            vm,
            program,
            fun.unwrap() as usize,
            steps.unwrap() as i32,
            t_start.unwrap(),
            t_end.unwrap(),
            fr.0,
            fr.1,
            to.0,
            to.1,
            mapping,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_path_circle(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut position: Option<(f32, f32)> = Some((0.0, 0.0));
    let mut radius: Option<f32> = Some(100.0);
    let mut steps: Option<f32> = Some(10.0);
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut fun: Option<i32> = Some(-1);
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(position, Keyword::Position, iname, value);
            read_float!(radius, Keyword::Radius, iname, value);
            read_float!(steps, Keyword::Steps, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_i32!(fun, Keyword::Fn, iname, value);
            read_kw!(mapping, Keyword::Mapping, iname, value);
        }
    }

    let pos = position.unwrap();
    let maybe_mapping = easing_from_keyword(mapping.unwrap());

    if let Some(mapping) = maybe_mapping {
        path_circular(
            vm,
            program,
            fun.unwrap() as usize,
            steps.unwrap() as i32,
            t_start.unwrap(),
            t_end.unwrap(),
            pos.0,
            pos.1,
            radius.unwrap(),
            mapping,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_path_spline(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut coords: Option<&Vec<Var>> = None;

    let mut steps: Option<f32> = Some(10.0);
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut fun: Option<i32> = Some(-1);
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(steps, Keyword::Steps, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_i32!(fun, Keyword::Fn, iname, value);
            read_kw!(mapping, Keyword::Mapping, iname, value);
        }
    }

    if let Some(coords_) = coords {
        let co = array_f32_6_from_vec(coords_);
        let maybe_mapping = easing_from_keyword(mapping.unwrap());

        if let Some(mapping) = maybe_mapping {
            path_spline(
                vm,
                program,
                fun.unwrap() as usize,
                steps.unwrap() as i32,
                t_start.unwrap(),
                t_end.unwrap(),
                co,
                mapping,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_path_bezier(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut coords: Option<&Vec<Var>> = None;

    let mut steps: Option<f32> = Some(10.0);
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut fun: Option<i32> = Some(-1);
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(steps, Keyword::Steps, iname, value);
            read_float!(t_start, Keyword::TStart, iname, value);
            read_float!(t_end, Keyword::TEnd, iname, value);
            read_i32!(fun, Keyword::Fn, iname, value);
            read_kw!(mapping, Keyword::Mapping, iname, value);
        }
    }

    if let Some(coords_) = coords {
        let co = array_f32_8_from_vec(coords_);
        let maybe_mapping = easing_from_keyword(mapping.unwrap());

        if let Some(mapping) = maybe_mapping {
            path_bezier(
                vm,
                program,
                fun.unwrap() as usize,
                steps.unwrap() as i32,
                t_start.unwrap(),
                t_end.unwrap(),
                &co,
                mapping,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_symmetry_vertical(
    vm: &mut Vm,
    program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut fun: Option<i32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_symmetry_vertical(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_symmetry_horizontal(
    vm: &mut Vm,
    program: &Program,
    num_args: usize,
) -> Result<Var> {
    let mut fun: Option<i32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_symmetry_horizontal(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_symmetry_4(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut fun: Option<i32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_symmetry_4(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_symmetry_8(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut fun: Option<i32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_symmetry_8(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_rotate(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut fun: Option<i32> = None;
    let mut copies: Option<f32> = Some(3.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
            read_float!(copies, Keyword::Copies, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_rotate(vm, program, fun_ as usize, copies.unwrap() as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn bind_repeat_mirrored(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let mut fun: Option<i32> = None;
    let mut copies: Option<f32> = Some(3.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_i32!(fun, Keyword::Fn, iname, value);
            read_float!(copies, Keyword::Copies, iname, value);
        }
    }

    if let Some(fun_) = fun {
        repeat_rotate_mirrored(vm, program, fun_ as usize, copies.unwrap() as usize)?;
    }

    Ok(Var::Bool(true))
}

#[cfg(test)]
mod tests {
    use crate::colour::ColourFormat;
    use crate::geometry::RENDER_PACKET_FLOAT_PER_VERTEX;
    use crate::vm::tests::*;
    use crate::vm::*;

    fn is_col_rgb(s: &str, r: f32, g: f32, b: f32, alpha: f32) {
        let mut vm = Vm::new();
        if let Var::Colour(fmt, e0, e1, e2, e3) = vm_exec(&mut vm, s) {
            assert_eq!(fmt, ColourFormat::Rgb);
            assert_eq!(e0, r);
            assert_eq!(e1, g);
            assert_eq!(e2, b);
            assert_eq!(e3, alpha);
        }
    }

    // get render packet 0's geometry length
    fn rp0_num_vertices(vm: &Vm, expected_num_vertices: usize) {
        assert_eq!(
            vm.get_render_packet_geo_len(0),
            expected_num_vertices * RENDER_PACKET_FLOAT_PER_VERTEX
        );
    }

    // #[test]
    fn dev_rendering_fns() {
        let mut vm = Vm::new();
        vm_run(&mut vm, "(line width: 33 from: [2 3] to: [400 500] colour: (col/rgb r: 0 g: 0.1 b: 0.2 alpha: 0.3))");
        // vm_run(&mut vm, "(line width: 0 from: [2 3] to: [400 500] brush: brush-b colour: (col/rgb r: 0 g: 0.1 b: 0.2 alpha: 0.3))");
        // vm_run(&mut vm, "(line brush: brush-b)");
        // vm_run(&mut vm, "(line brush: brush-b) (rect width: 10 height: 30)");

        let res = vm.top_stack_value().unwrap();
        if let Var::Debug(s) = res {
            assert_eq!(s, "x");
        } else {
            assert_eq!(false, true);
        }

        rp0_num_vertices(&vm, 4);
    }

    #[test]
    fn test_probe() {
        is_debug_str("(probe scalar: 0.4)", "0.4");
        is_debug_str(
            "(probe scalar: 0.4) (probe scalar: 0.7) (probe scalar: 0.9)",
            "0.4 0.7 0.9",
        );
    }

    #[test]
    fn test_bind_col_rgb() {
        is_col_rgb(
            "(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha: 0.4)",
            0.1,
            0.2,
            0.3,
            0.4,
        );
    }

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
