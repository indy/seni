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

use crate::colour::*;
use crate::compiler::Program;
use crate::ease::easing_from_keyword;
use crate::error::{Error, Result};
use crate::focal;
use crate::interp;
use crate::keywords::Keyword;
use crate::mathutil;
use crate::packable::{Mule, Packable};
use crate::path;
use crate::prng;
use crate::repeat;
use crate::uvmapper::BrushType;
use crate::vm::{Var, Vm};

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use strum_macros::{Display, EnumIter, EnumString};

pub type NativeCallback = fn(&mut Vm, &Program, usize) -> Result<Var>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString, EnumIter)]
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

pub fn build_native_fn_hash() -> HashMap<Native, NativeCallback> {
    let mut h: HashMap<Native, NativeCallback> = HashMap::new();

    // --------------------------------------------------
    // misc
    // --------------------------------------------------
    // BIND("debug/print", debug_print);
    h.insert(Native::Nth, nth);
    h.insert(Native::VectorLength, vector_length);
    h.insert(Native::Probe, probe);
    // map (todo)

    // --------------------------------------------------
    // shapes
    // --------------------------------------------------
    h.insert(Native::Line, line);
    h.insert(Native::Rect, rect);
    h.insert(Native::Circle, circle);
    h.insert(Native::CircleSlice, circle_slice);
    h.insert(Native::Poly, poly);
    h.insert(Native::Quadratic, quadratic);
    h.insert(Native::Bezier, bezier);
    h.insert(Native::BezierBulging, bezier_bulging);
    h.insert(Native::StrokedBezier, stroked_bezier);
    h.insert(Native::StrokedBezierRect, stroked_bezier_rect);

    // --------------------------------------------------
    // transforms
    // --------------------------------------------------
    h.insert(Native::Translate, translate);
    h.insert(Native::Rotate, rotate);
    h.insert(Native::Scale, scale);

    // --------------------------------------------------
    // colour
    // --------------------------------------------------
    h.insert(Native::ColConvert, col_convert);
    h.insert(Native::ColRGB, col_rgb);
    h.insert(Native::ColHSL, col_hsl);
    h.insert(Native::ColHSLuv, col_hsluv);
    h.insert(Native::ColHSV, col_hsv);
    h.insert(Native::ColLAB, col_lab);
    h.insert(Native::ColComplementary, col_complementary);
    h.insert(Native::ColSplitComplementary, col_split_complementary);
    h.insert(Native::ColAnalagous, col_analagous);
    h.insert(Native::ColTriad, col_triad);
    h.insert(Native::ColDarken, col_darken);
    h.insert(Native::ColLighten, col_lighten);
    h.insert(Native::ColSetAlpha, col_set_alpha);
    h.insert(Native::ColGetAlpha, col_get_alpha);
    h.insert(Native::ColSetR, col_set_r);
    h.insert(Native::ColGetR, col_get_r);
    h.insert(Native::ColSetG, col_set_g);
    h.insert(Native::ColGetG, col_get_g);
    h.insert(Native::ColSetB, col_set_b);
    h.insert(Native::ColGetB, col_get_b);
    h.insert(Native::ColSetH, col_set_h);
    h.insert(Native::ColGetH, col_get_h);
    h.insert(Native::ColSetS, col_set_s);
    h.insert(Native::ColGetS, col_get_s);
    h.insert(Native::ColSetL, col_set_l);
    h.insert(Native::ColGetL, col_get_l);
    h.insert(Native::ColSetA, col_set_a);
    h.insert(Native::ColGetA, col_get_a);
    h.insert(Native::ColSetV, col_set_v);
    h.insert(Native::ColGetV, col_get_v);
    h.insert(Native::ColBuildProcedural, col_build_procedural);
    // BIND("col/build-bezier", col_build_bezier);
    h.insert(Native::ColValue, col_value);

    // --------------------------------------------------
    // math
    // --------------------------------------------------
    h.insert(Native::MathDistance, math_distance);
    h.insert(Native::MathNormal, math_normal);
    h.insert(Native::MathClamp, math_clamp);
    h.insert(Native::MathRadiansDegrees, math_radians_to_degrees);
    h.insert(Native::MathCos, math_cos);
    h.insert(Native::MathSin, math_sin);

    // --------------------------------------------------
    // prng
    // --------------------------------------------------
    h.insert(Native::PrngBuild, prng_build);
    h.insert(Native::PrngValues, prng_values);
    h.insert(Native::PrngValue, prng_value);
    h.insert(Native::PrngPerlin, prng_perlin);

    // --------------------------------------------------
    // interp
    // --------------------------------------------------
    h.insert(Native::InterpBuild, interp_build);
    h.insert(Native::InterpValue, interp_value);
    h.insert(Native::InterpCos, interp_cos);
    h.insert(Native::InterpSin, interp_sin);
    h.insert(Native::InterpBezier, interp_bezier);
    h.insert(Native::InterpBezierTangent, interp_bezier_tangent);
    h.insert(Native::InterpRay, interp_ray);
    h.insert(Native::InterpLine, interp_line);
    h.insert(Native::InterpCircle, interp_circle);

    // --------------------------------------------------
    // path
    // --------------------------------------------------
    h.insert(Native::PathLinear, path_linear);
    h.insert(Native::PathCircle, path_circle);
    h.insert(Native::PathSpline, path_spline);
    h.insert(Native::PathBezier, path_bezier);

    // --------------------------------------------------
    // repeat
    // --------------------------------------------------
    h.insert(Native::RepeatSymmetryVertical, repeat_symmetry_vertical);
    h.insert(Native::RepeatSymmetryHorizontal, repeat_symmetry_horizontal);
    h.insert(Native::RepeatSymmetry4, repeat_symmetry_4);
    h.insert(Native::RepeatSymmetry8, repeat_symmetry_8);
    h.insert(Native::RepeatRotate, repeat_rotate);
    h.insert(Native::RepeatRotateMirrored, repeat_mirrored);

    // --------------------------------------------------
    // focal
    // --------------------------------------------------
    h.insert(Native::FocalBuildPoint, focal_build_point);
    h.insert(Native::FocalBuildHLine, focal_build_hline);
    h.insert(Native::FocalBuildVLine, focal_build_vline);
    h.insert(Native::FocalValue, focal_value);

    // --------------------------------------------------
    // gen
    // --------------------------------------------------
    h.insert(Native::GenStrayInt, gen_stray_int);
    h.insert(Native::GenStray, gen_stray);
    h.insert(Native::GenStray2D, gen_stray_2d);
    h.insert(Native::GenStray3D, gen_stray_3d);
    h.insert(Native::GenStray4D, gen_stray_4d);
    h.insert(Native::GenInt, gen_int);
    h.insert(Native::GenScalar, gen_scalar);
    h.insert(Native::Gen2D, gen_2d);
    h.insert(Native::GenSelect, gen_select);
    h.insert(Native::GenCol, gen_col);

    h
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

fn read_col(iname: i32, value: &Var, keyword: Keyword) -> Option<(Colour)> {
    if iname == keyword as i32 {
        if let Var::Colour(col) = value {
            return Some(*col);
        }
    }
    None
}

fn read_interp(iname: i32, value: &Var, keyword: Keyword) -> Option<interp::InterpStateStruct> {
    if iname == keyword as i32 {
        if let Var::InterpState(interp_state) = value {
            return Some(interp_state.clone());
        }
    }
    None
}

fn read_proc_colour(iname: i32, value: &Var, keyword: Keyword) -> Option<ProcColourStateStruct> {
    if iname == keyword as i32 {
        if let Var::ProcColourState(proc_colour_state) = value {
            return Some(proc_colour_state.clone());
        }
    }
    None
}

fn read_focal(iname: i32, value: &Var, keyword: Keyword) -> Option<focal::FocalStateStruct> {
    if iname == keyword as i32 {
        if let Var::FocalState(focal_state) = value {
            return Some(focal_state.clone());
        }
    }
    None
}

fn read_prng(iname: i32, value: &Var, keyword: Keyword) -> Option<RefMut<prng::PrngStateStruct>> {
    if iname == keyword as i32 {
        if let Var::PrngState(prng_state) = value {
            return Some(prng_state.borrow_mut());
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
macro_rules! read_interp {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_interp(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_proc_colour {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_proc_colour(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_focal {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_focal(*$in, $v, $kw).or($i);
    };
}
macro_rules! read_prng {
    ($i:ident, $kw:expr, $in:ident, $v:ident) => {
        $i = read_prng(*$in, $v, $kw).or($i);
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

pub fn nth(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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
            read_float_as_usize!(n, Keyword::N, iname, value);
        }
    }

    if let Some(from) = from {
        if let Var::Vector(vs) = from {
            if let Some(nth) = vs.get(n.unwrap_or(0)) {
                return Ok(nth.clone());
            } else {
                return Err(Error::Bind("nth: n out of range".to_string()));
            }
        } else if let Var::V2D(a, b) = from {
            match n.unwrap_or(0) {
                0 => return Ok(Var::Float(*a)),
                1 => return Ok(Var::Float(*b)),
                _ => return Err(Error::Bind("nth: n out of range".to_string())),
            }
        }
    }

    Err(Error::Bind(
        "nth requires vector argument in 'from'".to_string(),
    ))
}

pub fn vector_length(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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
        "vector_length requires vector argument".to_string(),
    ))
}

pub fn probe(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut from: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut to: Option<(f32, f32)> = Some((900.0, 900.0));
    let mut from_colour: Option<Colour> = None;
    let mut to_colour: Option<Colour> = None;
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("line matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    let from_col = if let Some(c) = from_colour {
        c
    } else {
        colour.unwrap()
    };

    let to_col = if let Some(c) = to_colour {
        c
    } else {
        colour.unwrap()
    };

    if let Ok(from_c) = from_col.convert(ColourFormat::Rgb) {
        if let Ok(to_c) = to_col.convert(ColourFormat::Rgb) {
            vm.geometry.render_line(
                matrix,
                from.unwrap(),
                to.unwrap(),
                width.unwrap(),
                &from_c,
                &to_c,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
    // Ok(Var::Debug(format!("s: {}", s).to_string()))
}

pub fn rect(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<Colour> = Some(Default::default());

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
        return Err(Error::Bind("line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.geometry.render_rect(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            &rgb_c,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    // if the radius has been defined then it overrides the width and height parameters
    if let Some(r) = radius {
        width = Some(r);
        height = Some(r);
    }

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.geometry.render_circle(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            &rgb_c,
            tessellation.unwrap() as usize,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn circle_slice(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut width: Option<f32> = Some(4.0);
    let mut height: Option<f32> = Some(10.0);
    let mut position: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    // if the radius has been defined then it overrides the width and height parameters
    if let Some(r) = radius {
        width = Some(r);
        height = Some(r);
    }

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.geometry.render_circle_slice(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            &rgb_c,
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

pub fn poly(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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
        return Err(Error::Bind("line matrix error".to_string()));
    };

    let uvm = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    if let Some(coords_) = coords {
        if let Some(colours_) = colours {
            vm.geometry.render_poly(matrix, coords_, colours_, uvm)?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn quadratic(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = None;
    let mut line_width_start: Option<f32> = Some(4.0);
    let mut line_width_end: Option<f32> = Some(4.0);
    let mut line_width_mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("bezier matrix error".to_string()));
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
        if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
            vm.geometry.render_quadratic(
                matrix,
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                &rgb_c,
                tessellation.unwrap() as usize,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bezier(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = None;
    let mut line_width_start: Option<f32> = Some(4.0);
    let mut line_width_end: Option<f32> = Some(4.0);
    let mut line_width_mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("bezier matrix error".to_string()));
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
        if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
            vm.geometry.render_bezier(
                matrix,
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                &rgb_c,
                tessellation.unwrap() as usize,
                uvm,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn bezier_bulging(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut line_width: Option<f32> = Some(4.0);
    let mut coords: Option<&Vec<Var>> = None;
    let mut t_start: Option<f32> = Some(0.0);
    let mut t_end: Option<f32> = Some(1.0);
    let mut tessellation: Option<f32> = Some(10.0);
    let mut colour: Option<Colour> = Some(Default::default());
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
        return Err(Error::Bind("bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    let co = array_f32_8_from_vec(coords.unwrap());

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.geometry.render_bezier_bulging(
            matrix,
            &co,
            line_width.unwrap(),
            t_start.unwrap(),
            t_end.unwrap(),
            &rgb_c,
            tessellation.unwrap() as usize,
            uvm,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn stroked_bezier(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut tessellation: Option<f32> = Some(15.0);
    let mut coords: Option<&Vec<Var>> = None;
    let mut stroke_tessellation: Option<f32> = Some(10.0);
    let mut stroke_noise: Option<f32> = Some(25.0);
    let mut stroke_line_width_start: Option<f32> = Some(1.0);
    let mut stroke_line_width_end: Option<f32> = Some(1.0);
    let mut colour: Option<Colour> = Some(Default::default());
    let mut colour_volatility: Option<f32> = Some(0.0);
    let mut seed: Option<f32> = Some(0.0);
    let mut line_width_mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_vector!(coords, Keyword::Coords, iname, value);
            read_float!(
                stroke_tessellation,
                Keyword::StrokeTessellation,
                iname,
                value
            );
            read_float!(stroke_noise, Keyword::StrokeNoise, iname, value);
            read_float!(
                stroke_line_width_start,
                Keyword::StrokeLineWidthStart,
                iname,
                value
            );
            read_float!(
                stroke_line_width_end,
                Keyword::StrokeLineWidthEnd,
                iname,
                value
            );
            read_col!(colour, Keyword::Colour, iname, value);
            read_float!(colour_volatility, Keyword::ColourVolatility, iname, value);
            read_float!(seed, Keyword::Seed, iname, value);
            read_kw!(line_width_mapping, Keyword::LineWidthMapping, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    let co = array_f32_8_from_vec(coords.unwrap());

    let maybe_mapping = easing_from_keyword(line_width_mapping.unwrap());
    if let Some(mapping) = maybe_mapping {
        if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
            vm.geometry.render_stroked_bezier(
                matrix,
                tessellation.unwrap() as usize,
                &co,
                stroke_tessellation.unwrap() as usize,
                stroke_noise.unwrap(),
                stroke_line_width_start.unwrap(),
                stroke_line_width_end.unwrap(),
                &rgb_c,
                colour_volatility.unwrap(),
                seed.unwrap(),
                mapping,
                uvm,
            )?;

            return Ok(Var::Bool(true));
        }
    }

    Ok(Var::Bool(false))
}

pub fn stroked_bezier_rect(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut position: Option<(f32, f32)> = Some((100.0, 100.0));
    let mut width: Option<f32> = Some(800.0);
    let mut height: Option<f32> = Some(600.0);
    let mut volatility: Option<f32> = Some(30.0);
    let mut overlap: Option<f32> = Some(0.0);
    let mut iterations: Option<f32> = Some(10.0);
    let mut seed: Option<f32> = Some(0.0);
    let mut tessellation: Option<f32> = Some(15.0);
    let mut stroke_tessellation: Option<f32> = Some(10.0);
    let mut stroke_noise: Option<f32> = Some(25.0);
    let mut colour: Option<Colour> = Some(Default::default());
    let mut colour_volatility: Option<f32> = Some(0.0);
    let mut brush: Option<BrushType> = Some(BrushType::Flat);
    let mut brush_subtype: Option<usize> = Some(0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(position, Keyword::Position, iname, value);
            read_float!(width, Keyword::Width, iname, value);
            read_float!(height, Keyword::Height, iname, value);
            read_float!(volatility, Keyword::Volatility, iname, value);
            read_float!(overlap, Keyword::Overlap, iname, value);
            read_float!(iterations, Keyword::Iterations, iname, value);
            read_float!(seed, Keyword::Seed, iname, value);
            read_float!(tessellation, Keyword::Tessellation, iname, value);
            read_float!(
                stroke_tessellation,
                Keyword::StrokeTessellation,
                iname,
                value
            );
            read_float!(stroke_noise, Keyword::StrokeNoise, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
            read_float!(colour_volatility, Keyword::ColourVolatility, iname, value);
            read_brush!(brush, Keyword::Brush, iname, value);
            read_float_as_usize!(brush_subtype, Keyword::BrushSubtype, iname, value);
        }
    }

    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Bind("bezier matrix error".to_string()));
    };

    let uvm = vm
        .mappings
        .get_uv_mapping(brush.unwrap(), brush_subtype.unwrap());

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.geometry.render_stroked_bezier_rect(
            matrix,
            position.unwrap(),
            width.unwrap(),
            height.unwrap(),
            volatility.unwrap(),
            overlap.unwrap(),
            iterations.unwrap(),
            seed.unwrap() as i32,
            tessellation.unwrap() as usize,
            stroke_tessellation.unwrap() as usize,
            stroke_noise.unwrap(),
            &rgb_c,
            colour_volatility.unwrap(),
            uvm,
        )?;

        return Ok(Var::Bool(true));
    }

    Ok(Var::Bool(false))
}

pub fn translate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn rotate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn scale(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn col_convert(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut format: Option<Keyword> = Some(Keyword::Rgb);
    let mut colour: Option<Colour> = Some(Default::default());

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_kw!(format, Keyword::Format, iname, value);
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    if let Some(fmt) = ColourFormat::from_keyword(format.unwrap()) {
        if let Some(colour) = colour {
            let col = colour.convert(fmt)?;
            return Ok(Var::Colour(col));
        }
    }

    Err(Error::Bind("col_convert".to_string()))
}

pub fn col_rgb(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    Ok(Var::Colour(Colour::new(
        ColourFormat::Rgb,
        r.unwrap(),
        g.unwrap(),
        b.unwrap(),
        alpha.unwrap(),
    )))
}

pub fn col_hsl(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsl,
        h.unwrap(),
        s.unwrap(),
        l.unwrap(),
        alpha.unwrap(),
    )))
}

pub fn col_hsluv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsluv,
        h.unwrap(),
        s.unwrap(),
        l.unwrap(),
        alpha.unwrap(),
    )))
}

pub fn col_hsv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsv,
        h.unwrap(),
        s.unwrap(),
        v.unwrap(),
        alpha.unwrap(),
    )))
}

pub fn col_lab(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    Ok(Var::Colour(Colour::new(
        ColourFormat::Lab,
        l.unwrap(),
        a.unwrap(),
        b.unwrap(),
        alpha.unwrap(),
    )))
}

pub fn col_complementary(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    if let Some(col) = colour {
        let c1 = col.complementary()?;
        return Ok(Var::Colour(c1));
    }

    Err(Error::Bind("col_complementary".to_string()))
}

pub fn col_split_complementary(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    if let Some(c) = colour {
        let (col1, col2) = c.split_complementary()?;
        return Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]));
    }

    Err(Error::Bind("col_split_complementary".to_string()))
}

pub fn col_analagous(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    if let Some(c) = colour {
        let (col1, col2) = c.analagous()?;
        return Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]));
    }

    Err(Error::Bind("col_analagous".to_string()))
}

pub fn col_triad(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    if let Some(c) = colour {
        let (col1, col2) = c.triad()?;
        return Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]));
    }

    Err(Error::Bind("col_triad".to_string()))
}

pub fn col_darken(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());
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

    if let Some(value) = val {
        if let Some(col) = colour {
            let darkened = col.darken(value)?;
            return Ok(Var::Colour(darkened));
        }
    }

    Err(Error::Bind("col_darken".to_string()))
}

pub fn col_lighten(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());
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

    if let Some(value) = val {
        if let Some(col) = colour {
            let lightened = col.lighten(value)?;
            return Ok(Var::Colour(lightened));
        }
    }

    Err(Error::Bind("col_lighten".to_string()))
}

pub fn col_set_elem(idx: usize, vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());
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

    let col = colour.unwrap();
    match idx {
        0 => Ok(Var::Colour(Colour::new(
            col.format,
            val.unwrap(),
            col.e1,
            col.e2,
            col.e3,
        ))),
        1 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            val.unwrap(),
            col.e2,
            col.e3,
        ))),
        2 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            col.e1,
            val.unwrap(),
            col.e3,
        ))),
        3 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            col.e1,
            col.e2,
            val.unwrap(),
        ))),
        _ => Err(Error::Bind("col_set_elem::idx out of range".to_string())),
    }
}

pub fn col_get_elem(idx: usize, vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut colour: Option<Colour> = Some(Default::default());
    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_col!(colour, Keyword::Colour, iname, value);
        }
    }

    let col = colour.unwrap();
    match idx {
        0 => Ok(Var::Float(col.e0)),
        1 => Ok(Var::Float(col.e1)),
        2 => Ok(Var::Float(col.e2)),
        3 => Ok(Var::Float(col.e3)),
        _ => Err(Error::Bind("col_get_elem::idx out of range".to_string())),
    }
}

pub fn col_set_alpha(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(3, vm, program, num_args)
}

pub fn col_get_alpha(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(3, vm, program, num_args)
}

pub fn col_set_r(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(0, vm, program, num_args)
}

pub fn col_get_r(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(0, vm, program, num_args)
}

pub fn col_set_g(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(1, vm, program, num_args)
}

pub fn col_get_g(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(1, vm, program, num_args)
}

pub fn col_set_b(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(2, vm, program, num_args)
}

pub fn col_get_b(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(2, vm, program, num_args)
}

pub fn col_set_h(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(0, vm, program, num_args)
}

pub fn col_get_h(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(0, vm, program, num_args)
}

pub fn col_set_s(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(1, vm, program, num_args)
}

pub fn col_get_s(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(1, vm, program, num_args)
}

pub fn col_set_l(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(2, vm, program, num_args)
}

pub fn col_get_l(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(2, vm, program, num_args)
}

pub fn col_set_a(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(1, vm, program, num_args)
}

pub fn col_get_a(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(1, vm, program, num_args)
}

pub fn col_set_v(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_set_elem(2, vm, program, num_args)
}

pub fn col_get_v(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    col_get_elem(2, vm, program, num_args)
}

fn to_f32_3(v: Option<&Vec<Var>>) -> Result<[f32; 3]> {
    if let Some(vecs) = v {
        if let Var::Float(a) = vecs[0] {
            if let Var::Float(b) = vecs[1] {
                if let Var::Float(c) = vecs[2] {
                    return Ok([a, b, c]);
                }
            }
        }
    }

    Err(Error::Bind("to_f32_3".to_string()))
}

pub fn col_build_procedural(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut preset: Option<Keyword> = None;
    let mut alpha: Option<f32> = Some(1.0);
    let mut a: Option<&Vec<Var>> = None;
    let mut b: Option<&Vec<Var>> = None;
    let mut c: Option<&Vec<Var>> = None;
    let mut d: Option<&Vec<Var>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_kw!(preset, Keyword::Preset, iname, value);
            read_float!(alpha, Keyword::Alpha, iname, value);
            read_vector!(a, Keyword::A, iname, value);
            read_vector!(b, Keyword::B, iname, value);
            read_vector!(c, Keyword::C, iname, value);
            read_vector!(d, Keyword::D, iname, value);
        }
    }

    if let Some(preset_keyword) = preset {
        if let Some(preset) = ColourPreset::from_keyword(preset_keyword) {
            let (a, b, c, d) = preset.get_preset();

            return Ok(Var::ProcColourState(ProcColourStateStruct {
                a,
                b,
                c,
                d,
                alpha: alpha.unwrap(),
            }));
        }
    }

    Ok(Var::ProcColourState(ProcColourStateStruct {
        a: to_f32_3(a)?,
        b: to_f32_3(b)?,
        c: to_f32_3(c)?,
        d: to_f32_3(d)?,
        alpha: alpha.unwrap(),
    }))
}

pub fn col_value(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<ProcColourStateStruct> = None;
    let mut t: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_proc_colour!(from, Keyword::From, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    if let Some(proc_colour) = from {
        let t = t.unwrap();
        let res = proc_colour.colour(t);

        return Ok(Var::Colour(res));
    }

    Err(Error::Bind("col_value".to_string()))
}

pub fn math_distance(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn math_normal(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn math_clamp(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn math_radians_to_degrees(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn math_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn math_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

pub fn prng_build(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut seed: Option<f32> = Some(1.0);
    let mut min: Option<f32> = Some(0.0);
    let mut max: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(seed, Keyword::Seed, iname, value);
            read_float!(min, Keyword::Min, iname, value);
            read_float!(max, Keyword::Max, iname, value);
        }
    }

    let prng_state_struct =
        prng::PrngStateStruct::new(seed.unwrap() as i32, min.unwrap(), max.unwrap());

    Ok(Var::PrngState(Rc::new(RefCell::new(prng_state_struct))))
}

pub fn prng_values(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut ref_mut_prng_state: Option<RefMut<prng::PrngStateStruct>> = None;
    let mut num: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(num, Keyword::Num, iname, value);
            read_prng!(ref_mut_prng_state, Keyword::From, iname, value);
        }
    }

    let mut vs: Vec<Var> = Vec::new();

    if let Some(ref mut prng_state) = ref_mut_prng_state {
        for _ in 0..(num.unwrap() as i32) {
            let f = prng_state.prng_f32_defined_range();
            vs.push(Var::Float(f))
        }
    }

    Ok(Var::Vector(vs))
}

pub fn prng_value(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut ref_mut_prng_state: Option<RefMut<prng::PrngStateStruct>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_prng!(ref_mut_prng_state, Keyword::From, iname, value);
        }
    }

    if let Some(ref mut prng_state) = ref_mut_prng_state {
        let f = prng_state.prng_f32_defined_range();
        return Ok(Var::Float(f));
    }

    Err(Error::Bind("prng_value".to_string()))
}

pub fn prng_perlin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut x: Option<f32> = Some(1.0);
    let mut y: Option<f32> = Some(1.0);
    let mut z: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(x, Keyword::X, iname, value);
            read_float!(y, Keyword::Y, iname, value);
            read_float!(z, Keyword::Z, iname, value);
        }
    }

    let res = prng::perlin(x.unwrap(), y.unwrap(), z.unwrap());

    Ok(Var::Float(res))
}

pub fn interp_build(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from_: Option<(f32, f32)> = Some((0.0, 1.0));
    let mut to_: Option<(f32, f32)> = Some((0.0, 100.0));
    let mut clamping_: Option<Keyword> = Some(Keyword::False);
    let mut mapping_: Option<Keyword> = Some(Keyword::Linear);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(from_, Keyword::From, iname, value);
            read_v2d!(to_, Keyword::To, iname, value);
            read_kw!(clamping_, Keyword::Clamping, iname, value);
            read_kw!(mapping_, Keyword::Mapping, iname, value);
        }
    }

    if let Some(mapping) = easing_from_keyword(mapping_.unwrap()) {
        let from = from_.unwrap();
        let to = to_.unwrap();
        let clamping = clamping_.unwrap() == Keyword::True;

        let from_m = mathutil::mc_m(from.0, 0.0, from.1, 1.0);
        let from_c = mathutil::mc_c(from.0, 0.0, from_m);
        let to_m = mathutil::mc_m(0.0, to.0, 1.0, to.1);
        let to_c = mathutil::mc_c(0.0, to.0, to_m);

        return Ok(Var::InterpState(interp::InterpStateStruct {
            from_m,
            to_m,
            from_c,
            to_c,
            to,
            clamping,
            mapping,
        }));
    }

    Err(Error::Bind("interp_build".to_string()))
}

pub fn interp_value(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<interp::InterpStateStruct> = None;
    let mut t: Option<f32> = Some(0.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_interp!(from, Keyword::From, iname, value);
            read_float!(t, Keyword::T, iname, value);
        }
    }

    if let Some(interp_state) = from {
        let t = t.unwrap();
        let res = interp_state.value(t);

        return Ok(Var::Float(res));
    }

    Err(Error::Bind("interp_value".to_string()))
}

pub fn interp_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    let res = interp::cos(amplitude.unwrap(), frequency.unwrap(), t.unwrap());

    Ok(Var::Float(res))
}

pub fn interp_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    let res = interp::sin(amplitude.unwrap(), frequency.unwrap(), t.unwrap());

    Ok(Var::Float(res))
}

pub fn interp_bezier(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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
    let (x, y) = interp::bezier(&co, t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn interp_bezier_tangent(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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
    let (x, y) = interp::bezier_tangent(&co, t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn interp_ray(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    let (x, y) = interp::ray(point.unwrap(), direction.unwrap(), t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn interp_line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

        let x = interp::scalar(from_.0, to_.0, mapping, clamping_, t_);
        let y = interp::scalar(from_.1, to_.1, mapping, clamping_, t_);

        return Ok(Var::V2D(x, y));
    }

    Err(Error::Bind("interp_line".to_string()))
}

pub fn interp_circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
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

    let (x, y) = interp::circle(position.unwrap(), radius.unwrap(), t.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn path_linear(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        path::linear(
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

pub fn path_circle(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        path::circular(
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

pub fn path_spline(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
            path::spline(
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

pub fn path_bezier(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
            path::bezier(
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

pub fn repeat_symmetry_vertical(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::symmetry_vertical(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_horizontal(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::symmetry_horizontal(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_4(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::symmetry_4(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_8(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::symmetry_8(vm, program, fun_ as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_rotate(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::rotate(vm, program, fun_ as usize, copies.unwrap() as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_mirrored(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
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
        repeat::rotate_mirrored(vm, program, fun_ as usize, copies.unwrap() as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn focal_build_generic(
    vm: &mut Vm,
    num_args: usize,
    focal_type: focal::FocalType,
) -> Result<Var> {
    let mut mapping: Option<Keyword> = Some(Keyword::Linear);
    let mut position: Option<(f32, f32)> = Some((0.0, 1.0));
    let mut distance: Option<f32> = Some(1.0);
    let mut transform_pos: Option<Keyword> = Some(Keyword::False);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_kw!(mapping, Keyword::Mapping, iname, value);
            read_v2d!(position, Keyword::Position, iname, value);
            read_float!(distance, Keyword::Distance, iname, value);
            read_kw!(transform_pos, Keyword::TransformPosition, iname, value);
        }
    }

    if let Some(mapping) = easing_from_keyword(mapping.unwrap()) {
        let position = position.unwrap();
        let distance = distance.unwrap();
        let transform_pos = transform_pos.unwrap() == Keyword::True;

        return Ok(Var::FocalState(focal::FocalStateStruct {
            focal_type,
            mapping,
            position,
            distance,
            transform_pos,
        }));
    }

    Err(Error::Bind("focal_build_generic".to_string()))
}

pub fn focal_build_point(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    focal_build_generic(vm, num_args, focal::FocalType::Point)
}

pub fn focal_build_hline(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    focal_build_generic(vm, num_args, focal::FocalType::HLine)
}

pub fn focal_build_vline(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    focal_build_generic(vm, num_args, focal::FocalType::VLine)
}

pub fn focal_value(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<focal::FocalStateStruct> = None;
    let mut position: Option<(f32, f32)> = Some((0.0, 0.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_focal!(from, Keyword::From, iname, value);
            read_v2d!(position, Keyword::Position, iname, value);
        }
    }

    if let Some(focal) = from {
        let position = position.unwrap();
        let res = focal.value(vm, position);

        return Ok(Var::Float(res));
    }

    Err(Error::Bind("focal_value".to_string()))
}

// NOTE: gen/stray-int should still parse values as
// float as sen scripts won't produce any real ints
//
pub fn gen_stray_int(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<f32> = Some(1.0);
    let mut by: Option<f32> = Some(0.2);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(from, Keyword::From, iname, value);
            read_float!(by, Keyword::By, iname, value);
        }
    }

    let from = from.unwrap();
    let by = mathutil::absf(by.unwrap());

    let value = vm.prng_state.prng_f32_range(from - by, from + by);

    Ok(Var::Float(value.floor()))
}

pub fn gen_stray(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut from: Option<f32> = Some(1.0);
    let mut by: Option<f32> = Some(0.2);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(from, Keyword::From, iname, value);
            read_float!(by, Keyword::By, iname, value);
        }
    }

    let from = from.unwrap();
    let by = mathutil::absf(by.unwrap());

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(from - by, from + by);

    Ok(Var::Float(value))
}

pub fn gen_stray_2d(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    if !vm.building_with_trait_within_vector {
        return Err(Error::Bind(
            "gen_stray_2d should always be called with vm.building_with_trait_within_vector"
                .to_string(),
        ));
    }

    let mut from: Option<(f32, f32)> = Some((10.0, 10.0));
    let mut by: Option<(f32, f32)> = Some((1.0, 1.0));

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_v2d!(from, Keyword::From, iname, value);
            read_v2d!(by, Keyword::By, iname, value);
        }
    }

    let from = from.unwrap();
    let by = by.unwrap();

    let index = vm.trait_within_vector_index;
    let by_index;
    let from_index;
    if index == 0 {
        by_index = mathutil::absf(by.0);
        from_index = from.0;
    } else if index == 1 {
        by_index = mathutil::absf(by.1);
        from_index = from.1;
    } else {
        return Err(Error::Bind(
            "gen_stray_2d invalid trait_within_vector_index value".to_string(),
        ));
    }

    // pick a scalar between min and max
    let value = vm
        .prng_state
        .prng_f32_range(from_index - by_index, from_index + by_index);

    Ok(Var::Float(value))
}

pub fn gen_stray_3d(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    if !vm.building_with_trait_within_vector {
        return Err(Error::Bind(
            "gen_stray_3d should always be called with vm.building_with_trait_within_vector"
                .to_string(),
        ));
    }

    let mut from: Option<&Vec<Var>> = None;
    let mut by: Option<&Vec<Var>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(from, Keyword::From, iname, value);
            read_vector!(by, Keyword::By, iname, value);
        }
    }

    if from.is_none() || by.is_none() {
        return Err(Error::Bind(
            "gen_stray_3d requires both from and by parameters".to_string(),
        ));
    }

    let index = vm.trait_within_vector_index;

    let from = if let Some(var) = from.unwrap().get(index) {
        Var::get_float_value(&var)?
    } else {
        return Err(Error::Bind(
            "gen_stray_3d requires both from and by parameters".to_string(),
        ));
    };

    let by = if let Some(var) = by.unwrap().get(index) {
        Var::get_float_value(&var)?
    } else {
        return Err(Error::Bind(
            "gen_stray_3d requires both from and by parameters".to_string(),
        ));
    };

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(from - by, from + by);

    Ok(Var::Float(value))
}

pub fn gen_stray_4d(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    if !vm.building_with_trait_within_vector {
        return Err(Error::Bind(
            "gen_stray_4d should always be called with vm.building_with_trait_within_vector"
                .to_string(),
        ));
    }

    let mut from: Option<&Vec<Var>> = None;
    let mut by: Option<&Vec<Var>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(from, Keyword::From, iname, value);
            read_vector!(by, Keyword::By, iname, value);
        }
    }

    if from.is_none() || by.is_none() {
        return Err(Error::Bind(
            "gen_stray_4d requires both from and by parameters".to_string(),
        ));
    }

    let index = vm.trait_within_vector_index;

    let from = if let Some(var) = from.unwrap().get(index) {
        Var::get_float_value(&var)?
    } else {
        return Err(Error::Bind(
            "gen_stray_4d requires both from and by parameters".to_string(),
        ));
    };

    let by = if let Some(var) = by.unwrap().get(index) {
        Var::get_float_value(&var)?
    } else {
        return Err(Error::Bind(
            "gen_stray_4d requires both from and by parameters".to_string(),
        ));
    };

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(from - by, from + by);

    Ok(Var::Float(value))
}

pub fn gen_int(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut min: Option<f32> = Some(0.0);
    let mut max: Option<f32> = Some(1000.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(min, Keyword::Min, iname, value);
            read_float!(max, Keyword::Max, iname, value);
        }
    }

    // pick a scalar between min and max
    let value = vm
        .prng_state
        .prng_f32_range(min.unwrap(), max.unwrap() + 1.0);

    // todo: c-version returned f32, is it ok to cast this to i32?
    Ok(Var::Float(value.floor()))
}

pub fn gen_scalar(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut min: Option<f32> = Some(0.0);
    let mut max: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(min, Keyword::Min, iname, value);
            read_float!(max, Keyword::Max, iname, value);
        }
    }

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(min.unwrap(), max.unwrap());

    Ok(Var::Float(value))
}

pub fn gen_2d(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut min: Option<f32> = Some(0.0);
    let mut max: Option<f32> = Some(1.0);

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(min, Keyword::Min, iname, value);
            read_float!(max, Keyword::Max, iname, value);
        }
    }

    let x = vm.prng_state.prng_f32_range(min.unwrap(), max.unwrap());
    let y = vm.prng_state.prng_f32_range(min.unwrap(), max.unwrap());

    Ok(Var::V2D(x, y))
}

pub fn gen_select(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    // 'from' parameter should be a list
    // i.e. (gen/select from: '(1 2 3 4 5))
    //
    // this prevents any confusion between a vector and vec_2d
    // e.g. (gen/select from: [1 2 3 4 5]) vs. (gen/select from: [1 2])

    let mut from: Option<&Vec<Var>> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_vector!(from, Keyword::From, iname, value);
        }
    }

    if let Some(from) = from {
        let index = vm.prng_state.prng_usize_range(0, from.len());
        Ok(from[index].clone())
    } else {
        Err(Error::Bind(
            "gen_select: no from parameter given".to_string(),
        ))
    }
}

pub fn gen_col(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let mut alpha: Option<f32> = None;

    let mut args_pointer = vm.sp - (num_args * 2);
    for _ in 0..num_args {
        let label = &vm.stack[args_pointer];
        let value = &vm.stack[args_pointer + 1];
        args_pointer += 2;

        if let Var::Int(iname) = label {
            read_float!(alpha, Keyword::Alpha, iname, value);
        }
    }

    let alpha = if let Some(alpha) = alpha {
        alpha
    } else {
        // no alpha was given so generate a random value
        vm.prng_state.prng_f32_range(0.0, 1.0)
    };

    Ok(Var::Colour(Colour::new(
        ColourFormat::Rgb,
        vm.prng_state.prng_f32_range(0.0, 1.0),
        vm.prng_state.prng_f32_range(0.0, 1.0),
        vm.prng_state.prng_f32_range(0.0, 1.0),
        alpha,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::colour::ColourFormat;
    use crate::geometry::RENDER_PACKET_FLOAT_PER_VERTEX;
    use crate::vm::tests::*;
    use crate::vm::*;

    fn is_col_rgb(s: &str, r: f32, g: f32, b: f32, alpha: f32) {
        let mut vm = Vm::new();
        if let Var::Colour(col) = vm_exec(&mut vm, s) {
            assert_eq!(col.format, ColourFormat::Rgb);
            assert_eq!(col.e0, r);
            assert_eq!(col.e1, g);
            assert_eq!(col.e2, b);
            assert_eq!(col.e3, alpha);
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
    fn test_native_pack() {
        let mut res: String = "".to_string();
        Native::ColGetAlpha.pack(&mut res).unwrap();
        assert_eq!("col/get-alpha", res);
    }

    #[test]
    fn test_native_unpack() {
        let (res, _rem) = Native::unpack("col/get-alpha").unwrap();
        assert_eq!(res, Native::ColGetAlpha);
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
    fn test_col_rgb() {
        is_col_rgb(
            "(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha: 0.4)",
            0.1,
            0.2,
            0.3,
            0.4,
        );
    }

    #[test]
    fn test_nth() {
        is_float("(define v [1 2 3 4]) (nth from: v n: 0)", 1.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 1)", 2.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 2)", 3.0);
        is_float("(define v [1 2 3 4]) (nth from: v n: 3)", 4.0);

        is_float("(define v [9 8]) (nth from: v n: 0)", 9.0);
        is_float("(define v [9 8]) (nth from: v n: 1)", 8.0);
    }

    #[test]
    fn test_vector_length() {
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
    fn test_math() {
        is_float("(math/clamp value: 3 min: 2 max: 5)", 3.0);
        is_float("(math/clamp value: 1 min: 2 max: 5)", 2.0);
        is_float("(math/clamp value: 8 min: 2 max: 5)", 5.0);

        is_float("(math/radians->degrees angle: 0.3)", 17.188734);

        is_float("(math/cos angle: 0.7)", 0.7648422);
        is_float("(math/sin angle: 0.9)", 0.7833269);
    }

}
