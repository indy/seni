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

use strum_macros::EnumString;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumString)]
pub enum Native {
    #[strum(serialize="UnreachableNativeStart")]
    NativeStart = Keyword::KeywordEnd as isize,

    // misc
    //
    #[strum(serialize="debug/print")]
    DebugPrint,
    #[strum(serialize="nth")]
    Nth,
    #[strum(serialize="vector/length")]
    VectorLength,


    // shapes
    //
    #[strum(serialize="line")]
    Line,
    #[strum(serialize="rect")]
    Rect,
    #[strum(serialize="circle")]
    Circle,
    #[strum(serialize="circle-slice")]
    CircleSlice,
    #[strum(serialize="poly")]
    Poly,
    #[strum(serialize="bezier")]
    Bezier,
    #[strum(serialize="bezier-bulging")]
    BezierBulging,
    #[strum(serialize="stroked-bezier")]
    StrokedBezier,
    #[strum(serialize="stroked-bezier-rect")]
    StrokedBezierRect,


    // transforms
    //
    #[strum(serialize="translate")]
    Translate,
    #[strum(serialize="rotate")]
    Rotate,
    #[strum(serialize="scale")]
    Scale,

    // colour
    //
    #[strum(serialize="col/convert")]
    ColConvert,
    #[strum(serialize="col/rgb")]
    ColRGB, // start of colour constructors
    #[strum(serialize="col/hsl")]
    ColHSL,
    #[strum(serialize="col/hsluv")]
    ColHSLuv,
    #[strum(serialize="col/hsv")]
    ColHSV,
    #[strum(serialize="col/lab")]
    ColLAB, // end of colour constructors
    #[strum(serialize="col/complementary")]
    ColComplementary,
    #[strum(serialize="col/split-complementary")]
    ColSplitComplementary,
    #[strum(serialize="col/analagous")]
    ColAnalagous,
    #[strum(serialize="col/triad")]
    ColTriad,
    #[strum(serialize="col/darken")]
    ColDarken,
    #[strum(serialize="col/lighten")]
    ColLighten,
    #[strum(serialize="col/set-alpha")]
    ColSetAlpha,
    #[strum(serialize="col/get-alpha")]
    ColGetAlpha,
    #[strum(serialize="col/set-r")]
    ColSetR,
    #[strum(serialize="col/get-r")]
    ColGetR,
    #[strum(serialize="col/set-g")]
    ColSetG,
    #[strum(serialize="col/get-g")]
    ColGetG,
    #[strum(serialize="col/set-b")]
    ColSetB,
    #[strum(serialize="col/get-b")]
    ColGetB,
    #[strum(serialize="col/set-h")]
    ColSetH,
    #[strum(serialize="col/get-h")]
    ColGetH,
    #[strum(serialize="col/set-s")]
    ColSetS,
    #[strum(serialize="col/get-s")]
    ColGetS,
    #[strum(serialize="col/set-l")]
    ColSetL,
    #[strum(serialize="col/get-l")]
    ColGetL,
    #[strum(serialize="col/set-a")]
    ColSetA,
    #[strum(serialize="col/get-a")]
    ColGetA,
    #[strum(serialize="col/set-v")]
    ColSetV,
    #[strum(serialize="col/get-v")]
    ColGetV,
    #[strum(serialize="col/build-procedural")]
    ColBuildProcedural,
    #[strum(serialize="col/build-bezier")]
    ColBuildBezier,
    #[strum(serialize="col/value")]
    ColValue,


    // math
    //
    #[strum(serialize="math/distance")]
    MathDistance,
    #[strum(serialize="math/normal")]
    MathNormal,
    #[strum(serialize="math/clamp")]
    MathClamp,
    #[strum(serialize="math/radians->degrees")]
    MathRadiansDegrees,
    #[strum(serialize="math/cos")]
    MathCos,
    #[strum(serialize="math/sin")]
    MathSin,


    // prng
    //
    #[strum(serialize="prng/build")]
    PrngBuild,
    #[strum(serialize="prng/values")]
    PrngValues,
    #[strum(serialize="prng/value")]
    PrngValue,
    #[strum(serialize="prng/perlin")]
    PrngPerlin,


    // interp
    //
    #[strum(serialize="interp/build")]
    InterpBuild,
    #[strum(serialize="interp/value")]
    InterpValue,
    #[strum(serialize="interp/cos")]
    InterpCos,
    #[strum(serialize="interp/sin")]
    InterpSin,
    #[strum(serialize="interp/bezier")]
    InterpBezier,
    #[strum(serialize="interp/bezier-tangent")]
    InterpBezierTangent,
    #[strum(serialize="interp/ray")]
    InterpRay,
    #[strum(serialize="interp/line")]
    InterpLine,
    #[strum(serialize="interp/circle")]
    InterpCircle,


    // path
    //
    #[strum(serialize="path/linear")]
    PathLinear,
    #[strum(serialize="path/circle")]
    PathCircle,
    #[strum(serialize="path/spline")]
    PathSpline,
    #[strum(serialize="path/bezier")]
    PathBezier,


    // repeat
    //
    #[strum(serialize="repeat/symmetry-vertical")]
    RepeatSymmetryVertical,
    #[strum(serialize="repeat/symmetry-horizontal")]
    RepeatSymmetryHorizontal,
    #[strum(serialize="repeat/symmetry-4")]
    RepeatSymmetry4,
    #[strum(serialize="repeat/symmetry-8")]
    RepeatSymmetry8,
    #[strum(serialize="repeat/rotate")]
    RepeatRotate,
    #[strum(serialize="repeat/rotate-mirrored")]
    RepeatRotateMirror,


    // focal
    //
    #[strum(serialize="focal/build-point")]
    FocalBuildPoint,
    #[strum(serialize="focal/build-vline")]
    FocalBuildVLine,
    #[strum(serialize="focal/build-hline")]
    FocalBuildHLine,
    #[strum(serialize="focal/value")]
    FocalValue,


    // gen
    //
    #[strum(serialize="gen/stray-int")]
    GenStrayInt,
    #[strum(serialize="gen/stray")]
    GenStray,
    #[strum(serialize="gen/stray-2d")]
    GenStray2D,
    #[strum(serialize="gen/stray-3d")]
    GenStray3D,
    #[strum(serialize="gen/stray-4d")]
    GenStray4D,
    #[strum(serialize="gen/int")]
    GenInt,
    #[strum(serialize="gen/scalar")]
    GenScalar,
    #[strum(serialize="gen/2d")]
    Gen2D,
    #[strum(serialize="gen/select")]
    GenSelect,
    #[strum(serialize="gen/col")]
    GenCol,

    #[strum(serialize="UnreachableNativeEnd")]
    NativeEnd,
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
