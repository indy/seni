// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::bitmap;
use crate::colour::{Colour, ColourFormat, ColourPreset, ProcColourStateStruct, ProcColourType};
use crate::context::Context;
use crate::ease::easing_from_keyword;
use crate::error::{Error, Result};
use crate::focal;
use crate::iname::Iname;
use crate::interp;
use crate::keywords::Keyword;
use crate::mathutil;
use crate::packable::{Mule, Packable};
use crate::path;
use crate::prng;
use crate::program::Program;
use crate::repeat;
use crate::uvmapper::BrushType;
use crate::vm::{StackPeek, Var, Vm};
use log::error;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString, EnumIter)]
pub enum Native {
    #[strum(serialize = "UnreachableNativeStart")]
    NativeStart = Keyword::KeywordEnd as isize,

    // // misc
    // //
    // #[strum(serialize = "debug/print")]
    // DebugPrint,
    #[strum(serialize = "nth")]
    Nth,
    #[strum(serialize = "vector/length")]
    VectorLength,
    #[strum(serialize = "probe")]
    Probe,
    #[strum(serialize = "meta")]
    Meta,
    #[strum(serialize = "get-x")]
    GetX,
    #[strum(serialize = "get-y")]
    GetY,

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

    // matrix transforms
    //
    #[strum(serialize = "__matrix_push")]
    MatrixPush, // special native function invoked by the compiler for on-matrix-stack
    #[strum(serialize = "__matrix_pop")]
    MatrixPop, // special native function invoked by the compiler for on-matrix-stack
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
    #[strum(serialize = "col/e0")]
    ColE0,
    #[strum(serialize = "col/e1")]
    ColE1,
    #[strum(serialize = "col/e2")]
    ColE2,
    #[strum(serialize = "col/alpha")]
    ColAlpha,
    #[strum(serialize = "col/set-e0")]
    ColSetE0,
    #[strum(serialize = "col/set-e1")]
    ColSetE1,
    #[strum(serialize = "col/set-e2")]
    ColSetE2,
    #[strum(serialize = "col/set-alpha")]
    ColSetAlpha,
    #[strum(serialize = "col/add-e0")]
    ColAddE0,
    #[strum(serialize = "col/add-e1")]
    ColAddE1,
    #[strum(serialize = "col/add-e2")]
    ColAddE2,
    #[strum(serialize = "col/add-alpha")]
    ColAddAlpha,
    #[strum(serialize = "col/build-procedural")]
    ColBuildProcedural,
    #[strum(serialize = "col/build-bezier")]
    ColBuildBezier,
    #[strum(serialize = "col/value")]
    ColValue,
    #[strum(serialize = "col/palette")]
    ColPalette,

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

    // bitmap
    //
    #[strum(serialize = "bitmap/each")]
    BitmapEach,
    #[strum(serialize = "bitmap/value")]
    BitmapValue,
    #[strum(serialize = "bitmap/width")]
    BitmapWidth,
    #[strum(serialize = "bitmap/height")]
    BitmapHeight,

    // masking
    //
    #[strum(serialize = "mask/set")]
    MaskSet,

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

// return a tuple
// .0 == input arguments as a vector of (name, default value) pairs
// .1 == how the native function affects the vm's stack in terms of opcode offset
//
pub fn parameter_info(native: Native) -> Result<(Vec<(Keyword, Var)>, i32)> {
    match native {
        // misc
        Native::Nth => nth_parameter_info(),
        Native::VectorLength => vector_length_parameter_info(),
        Native::Probe => probe_parameter_info(),
        Native::Meta => meta_parameter_info(),
        Native::GetX => get_x_parameter_info(),
        Native::GetY => get_y_parameter_info(),
        // shapes
        Native::Line => line_parameter_info(),
        Native::Rect => rect_parameter_info(),
        Native::Circle => circle_parameter_info(),
        Native::CircleSlice => circle_slice_parameter_info(),
        Native::Poly => poly_parameter_info(),
        Native::Quadratic => quadratic_parameter_info(),
        Native::Bezier => bezier_parameter_info(),
        Native::BezierBulging => bezier_bulging_parameter_info(),
        Native::StrokedBezier => stroked_bezier_parameter_info(),
        // transforms
        Native::MatrixPush => matrix_push_parameter_info(),
        Native::MatrixPop => matrix_pop_parameter_info(),
        Native::Translate => translate_parameter_info(),
        Native::Rotate => rotate_parameter_info(),
        Native::Scale => scale_parameter_info(),
        // colour
        Native::ColConvert => col_convert_parameter_info(),
        Native::ColRGB => col_rgb_parameter_info(),
        Native::ColHSL => col_hsl_parameter_info(),
        Native::ColHSLuv => col_hsluv_parameter_info(),
        Native::ColHSV => col_hsv_parameter_info(),
        Native::ColLAB => col_lab_parameter_info(),
        Native::ColComplementary => col_complementary_parameter_info(),
        Native::ColSplitComplementary => col_split_complementary_parameter_info(),
        Native::ColAnalagous => col_analagous_parameter_info(),
        Native::ColTriad => col_triad_parameter_info(),
        Native::ColDarken => common_colour_value_parameter_info(),
        Native::ColLighten => common_colour_value_parameter_info(),
        Native::ColE0 => col_get_parameter_info(),
        Native::ColE1 => col_get_parameter_info(),
        Native::ColE2 => col_get_parameter_info(),
        Native::ColAlpha => col_get_parameter_info(),
        Native::ColSetE0 => col_set_parameter_info(),
        Native::ColSetE1 => col_set_parameter_info(),
        Native::ColSetE2 => col_set_parameter_info(),
        Native::ColSetAlpha => col_set_parameter_info(),
        Native::ColAddE0 => col_add_parameter_info(),
        Native::ColAddE1 => col_add_parameter_info(),
        Native::ColAddE2 => col_add_parameter_info(),
        Native::ColAddAlpha => col_add_parameter_info(),
        Native::ColBuildProcedural => col_build_procedural_parameter_info(),
        Native::ColBuildBezier => col_build_bezier_parameter_info(),
        Native::ColValue => col_value_parameter_info(),
        Native::ColPalette => col_palette_parameter_info(),
        // math
        Native::MathDistance => math_distance_parameter_info(),
        Native::MathNormal => math_normal_parameter_info(),
        Native::MathClamp => math_clamp_parameter_info(),
        Native::MathRadiansDegrees => math_radians_degrees_parameter_info(),
        Native::MathCos => math_cos_parameter_info(),
        Native::MathSin => math_sin_parameter_info(),
        // prng
        Native::PrngBuild => prng_build_parameter_info(),
        Native::PrngValues => prng_values_parameter_info(),
        Native::PrngValue => prng_value_parameter_info(),
        Native::PrngPerlin => prng_perlin_parameter_info(),
        // interp
        Native::InterpBuild => interp_build_parameter_info(),
        Native::InterpValue => interp_value_parameter_info(),
        Native::InterpCos => interp_cos_parameter_info(),
        Native::InterpSin => interp_sin_parameter_info(),
        Native::InterpBezier => interp_bezier_parameter_info(),
        Native::InterpBezierTangent => interp_bezier_tangent_parameter_info(),
        Native::InterpRay => interp_ray_parameter_info(),
        Native::InterpLine => interp_line_parameter_info(),
        Native::InterpCircle => interp_circle_parameter_info(),
        // path
        Native::PathLinear => path_linear_parameter_info(),
        Native::PathCircle => path_circle_parameter_info(),
        Native::PathSpline => path_spline_parameter_info(),
        Native::PathBezier => path_bezier_parameter_info(),
        // repeat
        Native::RepeatSymmetryVertical => repeat_symmetry_vertical_parameter_info(),
        Native::RepeatSymmetryHorizontal => repeat_symmetry_horizontal_parameter_info(),
        Native::RepeatSymmetry4 => repeat_symmetry_4_parameter_info(),
        Native::RepeatSymmetry8 => repeat_symmetry_8_parameter_info(),
        Native::RepeatRotate => repeat_rotate_parameter_info(),
        Native::RepeatRotateMirrored => repeat_rotate_mirrored_parameter_info(),
        // focal
        Native::FocalBuildPoint => focal_build_generic_parameter_info(),
        Native::FocalBuildVLine => focal_build_generic_parameter_info(),
        Native::FocalBuildHLine => focal_build_generic_parameter_info(),
        Native::FocalValue => focal_value_parameter_info(),
        // bitmap
        Native::BitmapEach => bitmap_each_parameter_info(),
        Native::BitmapValue => bitmap_value_parameter_info(),
        Native::BitmapWidth => bitmap_width_parameter_info(),
        Native::BitmapHeight => bitmap_height_parameter_info(),
        // masking
        Native::MaskSet => mask_set_parameter_info(),
        // gen
        Native::GenStrayInt => gen_stray_int_parameter_info(),
        Native::GenStray => gen_stray_parameter_info(),
        Native::GenStray2D => gen_stray_2d_parameter_info(),
        Native::GenStray3D => gen_stray_3d_parameter_info(),
        Native::GenStray4D => gen_stray_4d_parameter_info(),
        Native::GenInt => gen_int_parameter_info(),
        Native::GenScalar => gen_scalar_parameter_info(),
        Native::Gen2D => gen_2d_parameter_info(),
        Native::GenSelect => gen_select_parameter_info(),
        Native::GenCol => gen_col_parameter_info(),
        _ => {
            error!("parameter_info");
            Err(Error::Native)
        }
    }
}

pub fn execute_native(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    native: Native,
) -> Result<Option<Var>> {
    match native {
        // misc
        Native::Nth => nth_execute(vm),
        Native::VectorLength => vector_length_execute(vm),
        Native::Probe => probe_execute(vm, context),
        Native::Meta => meta_execute(vm, context),
        Native::GetX => get_x_execute(vm, context),
        Native::GetY => get_y_execute(vm, context),
        // shapes
        Native::Line => line_execute(vm, context),
        Native::Rect => rect_execute(vm, context),
        Native::Circle => circle_execute(vm, context),
        Native::CircleSlice => circle_slice_execute(vm, context),
        Native::Poly => poly_execute(vm, context),
        Native::Quadratic => quadratic_execute(vm, context),
        Native::Bezier => bezier_execute(vm, context),
        Native::BezierBulging => bezier_bulging_execute(vm, context),
        Native::StrokedBezier => stroked_bezier_execute(vm, context),
        // transforms
        Native::MatrixPush => matrix_push_execute(vm, context),
        Native::MatrixPop => matrix_pop_execute(vm, context),
        Native::Translate => translate_execute(vm, context),
        Native::Rotate => rotate_execute(vm, context),
        Native::Scale => scale_execute(vm, context),
        // colours
        Native::ColConvert => col_convert_execute(vm),
        Native::ColRGB => col_rgb_execute(vm),
        Native::ColHSL => col_hsl_execute(vm),
        Native::ColHSLuv => col_hsluv_execute(vm),
        Native::ColHSV => col_hsv_execute(vm),
        Native::ColLAB => col_lab_execute(vm),
        Native::ColComplementary => col_complementary_execute(vm),
        Native::ColSplitComplementary => col_split_complementary_execute(vm),
        Native::ColAnalagous => col_analagous_execute(vm),
        Native::ColTriad => col_triad_execute(vm),
        Native::ColDarken => col_darken_execute(vm),
        Native::ColLighten => col_lighten_execute(vm),
        Native::ColE0 => col_get_execute(vm, 0),
        Native::ColE1 => col_get_execute(vm, 1),
        Native::ColE2 => col_get_execute(vm, 2),
        Native::ColAlpha => col_get_execute(vm, 3),
        Native::ColSetE0 => col_set_execute(vm, 0),
        Native::ColSetE1 => col_set_execute(vm, 1),
        Native::ColSetE2 => col_set_execute(vm, 2),
        Native::ColSetAlpha => col_set_execute(vm, 3),
        Native::ColAddE0 => col_add_execute(vm, 0),
        Native::ColAddE1 => col_add_execute(vm, 1),
        Native::ColAddE2 => col_add_execute(vm, 2),
        Native::ColAddAlpha => col_add_execute(vm, 3),
        Native::ColBuildProcedural => col_build_procedural_execute(vm),
        Native::ColBuildBezier => col_build_bezier_execute(vm),
        Native::ColValue => col_value_execute(vm),
        Native::ColPalette => col_palette_execute(vm),
        // math
        Native::MathDistance => math_distance_execute(vm),
        Native::MathNormal => math_normal_execute(vm),
        Native::MathClamp => math_clamp_execute(vm),
        Native::MathRadiansDegrees => math_radians_degrees_execute(vm),
        Native::MathCos => math_cos_execute(vm),
        Native::MathSin => math_sin_execute(vm),
        // prng
        Native::PrngBuild => prng_build_execute(vm),
        Native::PrngValues => prng_values_execute(vm),
        Native::PrngValue => prng_value_execute(vm),
        Native::PrngPerlin => prng_perlin_execute(vm),
        // interp
        Native::InterpBuild => interp_build_execute(vm),
        Native::InterpValue => interp_value_execute(vm),
        Native::InterpCos => interp_cos_execute(vm),
        Native::InterpSin => interp_sin_execute(vm),
        Native::InterpBezier => interp_bezier_execute(vm),
        Native::InterpBezierTangent => interp_bezier_tangent_execute(vm),
        Native::InterpRay => interp_ray_execute(vm),
        Native::InterpLine => interp_line_execute(vm),
        Native::InterpCircle => interp_circle_execute(vm),
        // path
        Native::PathLinear => path_linear_execute(vm, context, program),
        Native::PathCircle => path_circle_execute(vm, context, program),
        Native::PathSpline => path_spline_execute(vm, context, program),
        Native::PathBezier => path_bezier_execute(vm, context, program),
        // repeat
        Native::RepeatSymmetryVertical => repeat_symmetry_vertical_execute(vm, context, program),
        Native::RepeatSymmetryHorizontal => {
            repeat_symmetry_horizontal_execute(vm, context, program)
        }
        Native::RepeatSymmetry4 => repeat_symmetry_4_execute(vm, context, program),
        Native::RepeatSymmetry8 => repeat_symmetry_8_execute(vm, context, program),
        Native::RepeatRotate => repeat_rotate_execute(vm, context, program),
        Native::RepeatRotateMirrored => repeat_rotate_mirrored_execute(vm, context, program),
        // focal
        Native::FocalBuildPoint => focal_build_point_execute(vm),
        Native::FocalBuildVLine => focal_build_vline_execute(vm),
        Native::FocalBuildHLine => focal_build_hline_execute(vm),
        Native::FocalValue => focal_value_execute(vm, context),
        // bitmap
        Native::BitmapEach => bitmap_each_execute(vm, context, program),
        Native::BitmapValue => bitmap_value_execute(vm, context, program),
        Native::BitmapWidth => bitmap_width_execute(vm, context, program),
        Native::BitmapHeight => bitmap_height_execute(vm, context, program),
        // masking
        Native::MaskSet => mask_set_execute(vm, context, program),
        // gen
        Native::GenStrayInt => gen_stray_int_execute(vm),
        Native::GenStray => gen_stray_execute(vm),
        Native::GenStray2D => gen_stray_2d_execute(vm),
        Native::GenStray3D => gen_stray_3d_execute(vm),
        Native::GenStray4D => gen_stray_4d_execute(vm),
        Native::GenInt => gen_int_execute(vm),
        Native::GenScalar => gen_scalar_execute(vm),
        Native::Gen2D => gen_2d_execute(vm),
        Native::GenSelect => gen_select_execute(vm),
        Native::GenCol => gen_col_execute(vm),

        _ => {
            error!("execute_native");
            Err(Error::Native)
        }
    }
}

pub fn name_to_native_hash() -> HashMap<Iname, Native> {
    let mut hm: HashMap<Iname, Native> = HashMap::new();

    for n in Native::iter() {
        hm.insert(Iname::from(n), n);
    }

    hm
}

fn is_arg_given(bits: i32, position: usize) -> bool {
    // note: the position value will corresspond to the stack_peek value.
    // stack peek values start at 1 rather than 0 (since the current stack
    // pointer points to the 'next' free stack location). therefore we'll
    // need to subtract one from the position value
    (bits & (1 << (position - 1))) == 0
}

// can't have this as a member of Vm thanks to the borrow checker
fn stack_peek_vars(stack: &[Var], sp: usize, offset: usize) -> Result<&Vec<Var>> {
    if let Var::Vector(vs) = &stack[sp - offset] {
        Ok(vs)
    } else {
        error!("expected Var::Vector");
        Err(Error::Native)
    }
}

fn read_brush(kw: Keyword) -> BrushType {
    match kw {
        Keyword::BrushFlat => BrushType::Flat,
        Keyword::BrushA => BrushType::A,
        Keyword::BrushB => BrushType::B,
        Keyword::BrushC => BrushType::C,
        Keyword::BrushD => BrushType::D,
        Keyword::BrushE => BrushType::E,
        Keyword::BrushF => BrushType::F,
        Keyword::BrushG => BrushType::G,
        _ => BrushType::Flat,
    }
}

fn stack_peek_proc_colour_state_struct(
    stack: &[Var],
    sp: usize,
    offset: usize,
) -> Result<&ProcColourStateStruct> {
    if let Var::ProcColourState(pcss) = &stack[sp - offset] {
        Ok(pcss)
    } else {
        error!("expected Var::ProcColourState");
        Err(Error::Native)
    }
}

fn stack_peek_interp_state_struct(
    stack: &[Var],
    sp: usize,
    offset: usize,
) -> Result<&interp::InterpStateStruct> {
    if let Var::InterpState(iss) = &stack[sp - offset] {
        Ok(iss)
    } else {
        error!("expected Var::InterpState");
        Err(Error::Native)
    }
}

fn stack_peek_focal_state_struct(
    stack: &[Var],
    sp: usize,
    offset: usize,
) -> Result<&focal::FocalStateStruct> {
    if let Var::FocalState(fss) = &stack[sp - offset] {
        Ok(fss)
    } else {
        error!("expected Var::FocalState");
        Err(Error::Native)
    }
}

fn ref_mut_prng_state_struct(
    stack: &[Var],
    sp: usize,
    offset: usize,
) -> Result<RefMut<prng::PrngStateStruct>> {
    if let Var::PrngState(prng_state_struct) = &stack[sp - offset] {
        Ok(prng_state_struct.borrow_mut())
    } else {
        error!("expected Var::PrngState");
        Err(Error::Native)
    }
}

fn to_f32_3(vecs: &[Var]) -> Result<[f32; 3]> {
    if let Var::Float(a) = vecs[0] {
        if let Var::Float(b) = vecs[1] {
            if let Var::Float(c) = vecs[2] {
                return Ok([a, b, c]);
            }
        }
    }

    error!("to_f32_3");
    Err(Error::Native)
}

fn nth_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::N, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn nth_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    // require a 'from' argument
    if !is_arg_given(default_mask, 1) {
        error!("nth requires from parameter");
        return Err(Error::Native);
    }
    // require an 'n' argument
    if !is_arg_given(default_mask, 2) {
        error!("nth requires n parameter");
        return Err(Error::Native);
    }

    let n: usize = vm.stack_peek(2)?;

    // from is either a Vector or a V2D
    let from_offset = 1;

    let res = match &vm.stack[vm.sp - from_offset] {
        Var::Vector(vs) => {
            if let Some(nth) = vs.get(n) {
                // optimisation: most of the values will be floats
                // or v2d so try and avoid calling clone
                match nth {
                    Var::Float(f) => Some(Var::Float(*f)),
                    Var::V2D(x, y) => Some(Var::V2D(*x, *y)),
                    // todo: try and get rid of the clone call to nth
                    _ => Some(nth.clone()),
                }
            } else {
                error!("nth: n out of range");
                return Err(Error::Native);
            }
        }
        Var::V2D(x, y) => match n {
            0 => Some(Var::Float(*x)),
            1 => Some(Var::Float(*y)),
            _ => {
                error!("nth indexing V2D out of range");
                return Err(Error::Native);
            }
        },
        _ => {
            error!("nth only accepts Vector or V2D in from parameter");
            return Err(Error::Native);
        }
    };

    Ok(res)
}

fn vector_length_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Bool(false))],
        // stack offset
        1,
    ))
}

fn vector_length_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;
    let vector_offset = 1;

    // require a 'vector' argument
    if !is_arg_given(default_mask, vector_offset) {
        error!("vector/length requires from parameter");
        return Err(Error::Native);
    }

    // vector is either a Vector or a V2D
    let res = match &vm.stack[vm.sp - vector_offset] {
        Var::Vector(vs) => Some(Var::Int(vs.len() as i32)),
        Var::V2D(_, _) => Some(Var::Int(2)),
        _ => {
            error!("vector/length only accepts Vector or V2D in 'from' parameter");
            return Err(Error::Native);
        }
    };

    Ok(res)
}

fn probe_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Scalar, Var::Float(0.0)),
            (Keyword::Vector, Var::V2D(0.0, 0.0)),
            (Keyword::WorldSpace, Var::V2D(0.0, 0.0)),
        ],
        // stack offset
        0,
    ))
}

fn probe_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(4)?;

    if is_arg_given(default_mask, 1) {
        let scalar: f32 = vm.stack_peek(1)?;
        vm.debug_str_append(&format!("{}", scalar));
    }

    if is_arg_given(default_mask, 2) {
        let (x, y): (f32, f32) = vm.stack_peek(2)?;
        vm.debug_str_append(&format!("({},{})", x, y));
    }

    if is_arg_given(default_mask, 3) {
        let (x, y): (f32, f32) = vm.stack_peek(3)?;
        if let Some(matrix) = context.matrix_stack.peek() {
            let (nx, ny) = matrix.transform_vec2(x, y);
            vm.debug_str_append(&format!("({},{})", nx, ny));
        }
    }

    Ok(None)
}

fn meta_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::LinearColourSpace, Var::Float(0.0))],
        // stack offset
        0,
    ))
}

fn meta_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if is_arg_given(default_mask, 1) {
        let scalar: f32 = vm.stack_peek(1)?;
        context.output_linear_colour_space = scalar > 0.0;
    }

    Ok(None)
}

fn get_x_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::V2D(0.0, 0.0))],
        // stack offset
        1,
    ))
}

// only works with V2D
//
fn get_x_execute(vm: &mut Vm, _context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    // require a 'from' argument
    if !is_arg_given(default_mask, 1) {
        error!("get-x requires from parameter");
        return Err(Error::Native);
    }

    let from: (f32, f32) = vm.stack_peek(1)?;
    let res = Some(Var::Float(from.0));

    Ok(res)
}

fn get_y_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::V2D(0.0, 0.0))],
        // stack offset
        1,
    ))
}

// only works with V2D
//
fn get_y_execute(vm: &mut Vm, _context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    // require a 'from' argument
    if !is_arg_given(default_mask, 1) {
        error!("get-y requires from parameter");
        return Err(Error::Native);
    }

    let from: (f32, f32) = vm.stack_peek(1)?;
    let res = Some(Var::Float(from.1));

    Ok(res)
}

fn line_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Width, Var::Float(4.0)),
            (Keyword::From, Var::V2D(10.0, 10.0)),
            (Keyword::To, Var::V2D(900.0, 900.0)),
            (Keyword::FromColour, Var::Colour(Default::default())),
            (Keyword::ToColour, Var::Colour(Default::default())),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn line_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let line_width: f32 = vm.stack_peek(1)?;
    let line_from: (f32, f32) = vm.stack_peek(2)?;
    let line_to: (f32, f32) = vm.stack_peek(3)?;
    let from_col: Colour = vm.stack_peek(4)?;
    let to_col: Colour = vm.stack_peek(5)?;
    let col: Colour = vm.stack_peek(6)?;
    let brush: Keyword = vm.stack_peek(7)?;
    let brush_subtype: usize = vm.stack_peek(8)?;

    let default_mask: i32 = vm.stack_peek(9)?;

    let brush_type = read_brush(brush);

    // if the from-colour and to-colour parameters are given
    if is_arg_given(default_mask, 4) && is_arg_given(default_mask, 5) {
        context.render_line(
            line_from,
            line_to,
            line_width,
            &from_col,
            &to_col,
            brush_type,
            brush_subtype,
        )?;
    } else {
        context.render_line(
            line_from,
            line_to,
            line_width,
            &col,
            &col,
            brush_type,
            brush_subtype,
        )?;
    };

    Ok(None)
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

fn rect_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let width: f32 = vm.stack_peek(1)?;
    let height: f32 = vm.stack_peek(2)?;
    let position: (f32, f32) = vm.stack_peek(3)?;
    let col: Colour = vm.stack_peek(4)?;

    context.render_rect(position, width, height, &col)?;

    Ok(None)
}

fn circle_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Width, Var::Float(4.0)),
            (Keyword::Height, Var::Float(10.0)),
            (Keyword::Position, Var::V2D(10.0, 10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Tessellation, Var::Float(10.0)),
            (Keyword::Radius, Var::Float(10.0)),
        ],
        // stack offset
        0,
    ))
}

fn circle_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let width: f32 = vm.stack_peek(1)?;
    let height: f32 = vm.stack_peek(2)?;
    let position: (f32, f32) = vm.stack_peek(3)?;
    let col: Colour = vm.stack_peek(4)?;
    let tessellation: usize = vm.stack_peek(5)?;
    let radius: f32 = vm.stack_peek(6)?;

    let default_mask: i32 = vm.stack_peek(7)?;

    if is_arg_given(default_mask, 6) {
        // given a radius value
        context.render_circle(position, radius, radius, &col, tessellation)?;
    } else {
        // radius was not explicitly specified
        context.render_circle(position, width, height, &col, tessellation)?;
    }

    Ok(None)
}

fn circle_slice_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Width, Var::Float(4.0)),
            (Keyword::Height, Var::Float(10.0)),
            (Keyword::Position, Var::V2D(10.0, 10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Tessellation, Var::Float(10.0)),
            (Keyword::Radius, Var::Float(10.0)),
            (Keyword::AngleStart, Var::Float(0.0)),
            (Keyword::AngleEnd, Var::Float(10.0)),
            (Keyword::InnerWidth, Var::Float(1.0)),
            (Keyword::InnerHeight, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn circle_slice_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let mut width: f32 = vm.stack_peek(1)?;
    let mut height: f32 = vm.stack_peek(2)?;
    let position: (f32, f32) = vm.stack_peek(3)?;
    let col: Colour = vm.stack_peek(4)?;
    let tessellation: usize = vm.stack_peek(5)?;
    let radius: f32 = vm.stack_peek(6)?;
    let angle_start: f32 = vm.stack_peek(7)?;
    let angle_end: f32 = vm.stack_peek(8)?;
    let inner_width: f32 = vm.stack_peek(9)?;
    let inner_height: f32 = vm.stack_peek(10)?;

    let default_mask: i32 = vm.stack_peek(11)?;

    if is_arg_given(default_mask, 6) {
        // given a radius value
        width = radius;
        height = radius;
    }

    context.render_circle_slice(
        position,
        width,
        height,
        &col,
        tessellation,
        angle_start,
        angle_end,
        inner_width,
        inner_height,
    )?;

    Ok(None)
}

fn poly_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::Colours, Var::Bool(false)),
        ],
        // stack offset
        0,
    ))
}

fn poly_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) || !is_arg_given(default_mask, 2) {
        error!("poly requires both coords and colours");
        return Err(Error::Native);
    }

    // code looks like this thanks to the borrow checker being anal
    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let colours = stack_peek_vars(&vm.stack, vm.sp, 2)?;

    context.render_poly(coords, colours)?;

    Ok(None)
}

fn quadratic_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::LineWidth, Var::Float(4.0)),
            (Keyword::LineWidthStart, Var::Float(4.0)),
            (Keyword::LineWidthEnd, Var::Float(4.0)),
            (Keyword::LineWidthMapping, Var::Keyword(Keyword::Linear)),
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Tessellation, Var::Float(10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn quadratic_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let line_width: f32 = vm.stack_peek(1)?;
    let mut line_width_start: f32 = vm.stack_peek(2)?;
    let mut line_width_end: f32 = vm.stack_peek(3)?;
    let line_width_mapping: Keyword = vm.stack_peek(4)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 5)?;
    let t_start: f32 = vm.stack_peek(6)?;
    let t_end: f32 = vm.stack_peek(7)?;
    let tessellation: usize = vm.stack_peek(8)?;
    let col: Colour = vm.stack_peek(9)?;
    let brush: Keyword = vm.stack_peek(10)?;
    let brush_subtype: usize = vm.stack_peek(11)?;

    let default_mask: i32 = vm.stack_peek(12)?;

    if !is_arg_given(default_mask, 5) {
        error!("quadratic requires coords");
        return Err(Error::Native);
    }

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };

    let brush_type = read_brush(brush);

    if let Some(mapping) = easing_from_keyword(line_width_mapping) {
        if is_arg_given(default_mask, 1) {
            // given a line width value
            line_width_start = line_width;
            line_width_end = line_width;
        }

        context.render_quadratic(
            &[x0, y0, x1, y1, x2, y2],
            line_width_start,
            line_width_end,
            mapping,
            t_start,
            t_end,
            &col,
            tessellation,
            brush_type,
            brush_subtype,
        )?;
    } else {
        error!("quadratic: invalid mapping");
        return Err(Error::Native);
    }

    Ok(None)
}

fn bezier_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::LineWidth, Var::Float(4.0)),
            (Keyword::LineWidthStart, Var::Float(4.0)),
            (Keyword::LineWidthEnd, Var::Float(4.0)),
            (Keyword::LineWidthMapping, Var::Keyword(Keyword::Linear)),
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Tessellation, Var::Float(10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn bezier_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let line_width: f32 = vm.stack_peek(1)?;
    let mut line_width_start: f32 = vm.stack_peek(2)?;
    let mut line_width_end: f32 = vm.stack_peek(3)?;
    let line_width_mapping: Keyword = vm.stack_peek(4)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 5)?;
    let t_start: f32 = vm.stack_peek(6)?;
    let t_end: f32 = vm.stack_peek(7)?;
    let tessellation: usize = vm.stack_peek(8)?;
    let col: Colour = vm.stack_peek(9)?;
    let brush: Keyword = vm.stack_peek(10)?;
    let brush_subtype: usize = vm.stack_peek(11)?;

    let default_mask: i32 = vm.stack_peek(12)?;

    if !is_arg_given(default_mask, 5) {
        error!("bezier requires coords");
        return Err(Error::Native);
    }

    let brush_type = read_brush(brush);

    if let Some(mapping) = easing_from_keyword(line_width_mapping) {
        let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
            (x, y)
        } else {
            error!("coords 0 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
            (x, y)
        } else {
            error!("coords 1 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
            (x, y)
        } else {
            error!("coords 2 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
            (x, y)
        } else {
            error!("coords 3 should be a Vec::V2D");
            return Err(Error::Native);
        };

        if is_arg_given(default_mask, 1) {
            // given a line width value
            line_width_start = line_width;
            line_width_end = line_width;
        }

        context.render_bezier(
            &[x0, y0, x1, y1, x2, y2, x3, y3],
            line_width_start,
            line_width_end,
            mapping,
            t_start,
            t_end,
            &col,
            tessellation,
            brush_type,
            brush_subtype,
        )?;
    } else {
        error!("bezier: invalid mapping");
        return Err(Error::Native);
    }

    Ok(None)
}

fn bezier_bulging_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::LineWidth, Var::Float(4.0)),
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Tessellation, Var::Float(10.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn bezier_bulging_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let line_width: f32 = vm.stack_peek(1)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 2)?;
    let t_start: f32 = vm.stack_peek(3)?;
    let t_end: f32 = vm.stack_peek(4)?;
    let tessellation: usize = vm.stack_peek(5)?;
    let col: Colour = vm.stack_peek(6)?;
    let brush: Keyword = vm.stack_peek(7)?;
    let brush_subtype: usize = vm.stack_peek(8)?;

    let default_mask: i32 = vm.stack_peek(9)?;

    if !is_arg_given(default_mask, 2) {
        error!("bezier_bulging requires coords");
        return Err(Error::Native);
    }

    let brush_type = read_brush(brush);

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        error!("coords 3 should be a Vec::V2D");
        return Err(Error::Native);
    };

    context.render_bezier_bulging(
        &[x0, y0, x1, y1, x2, y2, x3, y3],
        line_width,
        t_start,
        t_end,
        &col,
        tessellation,
        brush_type,
        brush_subtype,
    )?;

    Ok(None)
}

fn stroked_bezier_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Tessellation, Var::Float(15.0)),
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::StrokeTessellation, Var::Float(10.0)),
            (Keyword::StrokeNoise, Var::Float(25.0)),
            (Keyword::StrokeLineWidthStart, Var::Float(1.0)),
            (Keyword::StrokeLineWidthEnd, Var::Float(1.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::ColourVolatility, Var::Float(0.0)),
            (Keyword::Seed, Var::Float(0.0)),
            (Keyword::LineWidthMapping, Var::Keyword(Keyword::Linear)),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn stroked_bezier_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let tessellation: usize = vm.stack_peek(1)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 2)?;
    let stroke_tessellation: usize = vm.stack_peek(3)?;
    let stroke_noise: f32 = vm.stack_peek(4)?;
    let stroke_line_width_start: f32 = vm.stack_peek(5)?;
    let stroke_line_width_end: f32 = vm.stack_peek(6)?;
    let col: Colour = vm.stack_peek(7)?;
    let col_volatility: f32 = vm.stack_peek(8)?;
    let seed: f32 = vm.stack_peek(9)?;
    let line_width_mapping: Keyword = vm.stack_peek(10)?;
    let brush: Keyword = vm.stack_peek(11)?;
    let brush_subtype: usize = vm.stack_peek(12)?;

    let default_mask: i32 = vm.stack_peek(13)?;

    if !is_arg_given(default_mask, 2) {
        error!("stroked bezier requires coords");
        return Err(Error::Native);
    }

    let brush_type = read_brush(brush);

    if let Some(mapping) = easing_from_keyword(line_width_mapping) {
        let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
            (x, y)
        } else {
            error!("coords 0 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
            (x, y)
        } else {
            error!("coords 1 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
            (x, y)
        } else {
            error!("coords 2 should be a Vec::V2D");
            return Err(Error::Native);
        };
        let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
            (x, y)
        } else {
            error!("coords 3 should be a Vec::V2D");
            return Err(Error::Native);
        };

        context.render_stroked_bezier(
            tessellation,
            &[x0, y0, x1, y1, x2, y2, x3, y3],
            stroke_tessellation,
            stroke_noise,
            stroke_line_width_start,
            stroke_line_width_end,
            &col,
            col_volatility,
            seed,
            mapping,
            brush_type,
            brush_subtype,
        )?
    } else {
        error!("stroked bezier: invalid mapping");
        return Err(Error::Native);
    }

    Ok(None)
}

fn matrix_push_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![],
        // stack offset
        0,
    ))
}

fn matrix_push_execute(_vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    context.matrix_stack.push();

    Ok(None)
}

fn matrix_pop_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![],
        // stack offset
        0,
    ))
}

fn matrix_pop_execute(_vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    context.matrix_stack.pop();

    Ok(None)
}

fn translate_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Vector, Var::V2D(0.0, 0.0))],
        // stack offset
        0,
    ))
}

fn translate_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let (x, y): (f32, f32) = vm.stack_peek(1)?;

    context.matrix_stack.translate(x, y);

    Ok(None)
}

fn rotate_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Angle, Var::Float(0.0))],
        // stack offset
        0,
    ))
}

fn rotate_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let angle: f32 = vm.stack_peek(1)?;

    context.matrix_stack.rotate(mathutil::deg_to_rad(angle));

    Ok(None)
}

fn scale_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Vector, Var::V2D(1.0, 1.0)),
            (Keyword::Scalar, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn scale_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let (x, y): (f32, f32) = vm.stack_peek(1)?;
    let scalar: f32 = vm.stack_peek(2)?;

    let default_mask: i32 = vm.stack_peek(3)?;

    if is_arg_given(default_mask, 2) {
        // scalar was specified in the script
        context.matrix_stack.scale(scalar, scalar);
    } else {
        context.matrix_stack.scale(x, y);
    }

    Ok(None)
}

fn col_convert_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Format, Var::Bool(false)),
            (Keyword::From, Var::Colour(Default::default())),
        ],
        // stack offset
        1,
    ))
}

fn col_convert_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let format: Keyword = vm.stack_peek(1)?;
    let col: Colour = vm.stack_peek(2)?;

    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("col/convert requires format argument");
        Err(Error::Native)
    } else if let Some(format) = ColourFormat::from_keyword(format) {
        let col = col.convert(format)?;
        Ok(Some(Var::Colour(col)))
    } else {
        error!("col/convert");
        Err(Error::Native)
    }
}

fn col_rgb_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::R, Var::Float(0.0)),
            (Keyword::G, Var::Float(0.0)),
            (Keyword::B, Var::Float(0.0)),
            (Keyword::Alpha, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_rgb_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let r: f32 = vm.stack_peek(1)?;
    let g: f32 = vm.stack_peek(2)?;
    let b: f32 = vm.stack_peek(3)?;
    let alpha: f32 = vm.stack_peek(4)?;

    Ok(Some(Var::Colour(Colour::new(
        ColourFormat::Rgb,
        r,
        g,
        b,
        alpha,
    ))))
}

fn col_hsl_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::H, Var::Float(0.0)),
            (Keyword::S, Var::Float(0.0)),
            (Keyword::L, Var::Float(0.0)),
            (Keyword::Alpha, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_hsl_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let h: f32 = vm.stack_peek(1)?;
    let s: f32 = vm.stack_peek(2)?;
    let l: f32 = vm.stack_peek(3)?;
    let alpha: f32 = vm.stack_peek(4)?;

    Ok(Some(Var::Colour(Colour::new(
        ColourFormat::Hsl,
        h,
        s,
        l,
        alpha,
    ))))
}

fn col_hsluv_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::H, Var::Float(0.0)),
            (Keyword::S, Var::Float(0.0)),
            (Keyword::L, Var::Float(0.0)),
            (Keyword::Alpha, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_hsluv_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let h: f32 = vm.stack_peek(1)?;
    let s: f32 = vm.stack_peek(2)?;
    let l: f32 = vm.stack_peek(3)?;
    let alpha: f32 = vm.stack_peek(4)?;

    let colour = Colour::new(ColourFormat::Hsluv, h, s, l, alpha);
    // error!("col/hsluv: {:?}", &colour);
    Ok(Some(Var::Colour(colour)))
}

fn col_hsv_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::H, Var::Float(0.0)),
            (Keyword::S, Var::Float(0.0)),
            (Keyword::V, Var::Float(0.0)),
            (Keyword::Alpha, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_hsv_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let h: f32 = vm.stack_peek(1)?;
    let s: f32 = vm.stack_peek(2)?;
    let v: f32 = vm.stack_peek(3)?;
    let alpha: f32 = vm.stack_peek(4)?;

    Ok(Some(Var::Colour(Colour::new(
        ColourFormat::Hsv,
        h,
        s,
        v,
        alpha,
    ))))
}

fn col_lab_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::L, Var::Float(0.0)),
            (Keyword::A, Var::Float(0.0)),
            (Keyword::B, Var::Float(0.0)),
            (Keyword::Alpha, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_lab_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let l: f32 = vm.stack_peek(1)?;
    let a: f32 = vm.stack_peek(2)?;
    let b: f32 = vm.stack_peek(3)?;
    let alpha: f32 = vm.stack_peek(4)?;

    Ok(Some(Var::Colour(Colour::new(
        ColourFormat::Lab,
        l,
        a,
        b,
        alpha,
    ))))
}

fn col_complementary_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Colour(Default::default()))],
        // stack offset
        1,
    ))
}

fn col_complementary_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;

    Ok(Some(Var::Colour(col.complementary()?)))
}

fn col_split_complementary_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Colour(Default::default()))],
        // stack offset
        1,
    ))
}

fn col_split_complementary_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let (col1, col2) = col.split_complementary()?;

    Ok(Some(Var::Vector(vec![
        Var::Colour(col1),
        Var::Colour(col2),
    ])))
}

fn col_analagous_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Colour(Default::default()))],
        // stack offset
        1,
    ))
}

fn col_analagous_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let (col1, col2) = col.analagous()?;

    Ok(Some(Var::Vector(vec![
        Var::Colour(col1),
        Var::Colour(col2),
    ])))
}

fn col_triad_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Colour(Default::default()))],
        // stack offset
        1,
    ))
}

fn col_triad_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let (col1, col2) = col.triad()?;

    Ok(Some(Var::Vector(vec![
        Var::Colour(col1),
        Var::Colour(col2),
    ])))
}

fn common_colour_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Colour(Default::default())),
            (Keyword::Value, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_darken_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let value: f32 = vm.stack_peek(2)?;

    Ok(Some(Var::Colour(col.darken(value)?)))
}

fn col_lighten_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let value: f32 = vm.stack_peek(2)?;

    Ok(Some(Var::Colour(col.lighten(value)?)))
}

fn col_get_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Colour(Default::default()))],
        // stack offset
        1,
    ))
}

fn col_get_execute(vm: &mut Vm, idx: usize) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;

    let res = match idx {
        0 => col.e0,
        1 => col.e1,
        2 => col.e2,
        3 => col.e3,
        _ => {
            error!("col_get_execute::idx out of range");
            return Err(Error::Native);
        }
    };

    Ok(Some(Var::Float(res)))
}

fn col_set_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Colour(Default::default())),
            (Keyword::Value, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_set_execute(vm: &mut Vm, idx: usize) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let value: f32 = vm.stack_peek(2)?;

    let res = match idx {
        0 => Colour::new(col.format, value, col.e1, col.e2, col.e3),
        1 => Colour::new(col.format, col.e0, value, col.e2, col.e3),
        2 => Colour::new(col.format, col.e0, col.e1, value, col.e3),
        3 => Colour::new(col.format, col.e0, col.e1, col.e2, value),
        _ => {
            error!("col_set_execute::idx out of range");
            return Err(Error::Native);
        }
    };

    Ok(Some(Var::Colour(res)))
}

fn col_add_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Colour(Default::default())),
            (Keyword::Value, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_add_execute(vm: &mut Vm, idx: usize) -> Result<Option<Var>> {
    let col: Colour = vm.stack_peek(1)?;
    let value: f32 = vm.stack_peek(2)?;

    let res = match idx {
        0 => Colour::new(col.format, col.e0 + value, col.e1, col.e2, col.e3),
        1 => Colour::new(col.format, col.e0, col.e1 + value, col.e2, col.e3),
        2 => Colour::new(col.format, col.e0, col.e1, col.e2 + value, col.e3),
        3 => Colour::new(col.format, col.e0, col.e1, col.e2, col.e3 + value),
        _ => {
            error!("col_add_execute::idx out of range");
            return Err(Error::Native);
        }
    };

    Ok(Some(Var::Colour(res)))
}

fn col_build_procedural_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Preset, Var::Keyword(Keyword::Robocop)),
            (Keyword::Alpha, Var::Float(1.0)),
            (Keyword::A, Var::Float(0.0)),
            (Keyword::B, Var::Float(0.0)),
            (Keyword::C, Var::Float(0.0)),
            (Keyword::D, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_build_procedural_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(7)?;

    let alpha: f32 = vm.stack_peek(2)?;

    let (a, b, c, d) = if is_arg_given(default_mask, 1) {
        // preset given
        let preset_kw: Keyword = vm.stack_peek(1)?;
        if let Some(preset) = ColourPreset::from_keyword(preset_kw) {
            preset.get_preset()
        } else {
            error!("col_build_procedural_execute");
            return Err(Error::Native);
        }
    } else if is_arg_given(default_mask, 3)
        && is_arg_given(default_mask, 4)
        && is_arg_given(default_mask, 5)
        && is_arg_given(default_mask, 6)
    {
        (
            to_f32_3(stack_peek_vars(&vm.stack, vm.sp, 3)?)?,
            to_f32_3(stack_peek_vars(&vm.stack, vm.sp, 4)?)?,
            to_f32_3(stack_peek_vars(&vm.stack, vm.sp, 5)?)?,
            to_f32_3(stack_peek_vars(&vm.stack, vm.sp, 6)?)?,
        )
    } else {
        error!("col_build_procedural_execute");
        return Err(Error::Native);
    };

    Ok(Some(Var::ProcColourState(ProcColourStateStruct {
        proc_colour_type: ProcColourType::ProceduralColour,
        a,
        b,
        c,
        d,
        alpha: [alpha, 0.0, 0.0, 0.0],
    })))
}

fn col_build_bezier_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::A, Var::Colour(Default::default())),
            (Keyword::B, Var::Colour(Default::default())),
            (Keyword::C, Var::Colour(Default::default())),
            (Keyword::D, Var::Colour(Default::default())),
        ],
        // stack offset
        1,
    ))
}

fn col_build_bezier_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let col_a: Colour = vm.stack_peek(1)?;
    let col_b: Colour = vm.stack_peek(2)?;
    let col_c: Colour = vm.stack_peek(3)?;
    let col_d: Colour = vm.stack_peek(4)?;

    Ok(Some(Var::ProcColourState(ProcColourStateStruct {
        proc_colour_type: ProcColourType::BezierColour,
        a: [col_a.e0, col_a.e1, col_a.e2],
        b: [col_b.e0, col_b.e1, col_b.e2],
        c: [col_c.e0, col_c.e1, col_c.e2],
        d: [col_d.e0, col_d.e1, col_d.e2],
        alpha: [col_a.e3, col_b.e3, col_c.e3, col_d.e3],
    })))
}

fn col_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn col_value_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("col_value_execute requires from parameter");
        return Err(Error::Native);
    }

    let from = stack_peek_proc_colour_state_struct(&vm.stack, vm.sp, 1)?;
    let t: f32 = vm.stack_peek(2)?;

    let res = from.colour(t);

    Ok(Some(Var::Colour(res)))
}

fn col_palette_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Index, Var::Float(0.0))],
        // stack offset
        1,
    ))
}

fn col_palette_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("col_palette_execute requires index parameter");
        return Err(Error::Native);
    }

    let index: f32 = vm.stack_peek(1)?;
    let palette = Colour::palette(index as usize)?;

    let mut vs: Vec<Var> = Vec::new();
    for colour in palette {
        vs.push(Var::Colour(colour));
    }

    Ok(Some(Var::Vector(vs)))
}

fn math_distance_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Vec1, Var::V2D(0.0, 0.0)),
            (Keyword::Vec2, Var::V2D(0.0, 0.0)),
        ],
        // stack offset
        1,
    ))
}

fn math_distance_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let (x1, y1): (f32, f32) = vm.stack_peek(1)?;
    let (x2, y2): (f32, f32) = vm.stack_peek(2)?;

    let distance = mathutil::distance_v2(x1, y1, x2, y2);

    Ok(Some(Var::Float(distance)))
}

fn math_normal_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Vec1, Var::V2D(0.0, 0.0)),
            (Keyword::Vec2, Var::V2D(0.0, 0.0)),
        ],
        // stack offset
        1,
    ))
}

fn math_normal_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let (x1, y1): (f32, f32) = vm.stack_peek(1)?;
    let (x2, y2): (f32, f32) = vm.stack_peek(2)?;

    let distance = mathutil::normal(x1, y1, x2, y2);

    Ok(Some(Var::V2D(distance.0, distance.1)))
}

fn math_clamp_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Float(0.0)),
            (Keyword::Min, Var::Float(0.0)),
            (Keyword::Max, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn math_clamp_execute(vm: &mut Vm) -> Result<Option<Var>> {
    // todo: try and move functions like this into ones that initially
    // create and return a function that takes a single argument.
    // e.g.
    // (define my-clamp (math/clamp-fn min: 0.0 max: 42.0))
    // (my-clamp val: 22)
    //
    // then optimize for single argument functions as these will be much faster to
    // parse
    //
    let value: f32 = vm.stack_peek(1)?;
    let min: f32 = vm.stack_peek(2)?;
    let max: f32 = vm.stack_peek(3)?;

    let clamped = mathutil::clamp(value, min, max);

    Ok(Some(Var::Float(clamped)))
}

fn math_radians_degrees_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Float(0.0))],
        // stack offset
        1,
    ))
}

fn math_radians_degrees_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let rad_angle: f32 = vm.stack_peek(1)?;

    let deg_angle = mathutil::rad_to_deg(rad_angle);

    Ok(Some(Var::Float(deg_angle)))
}

fn math_cos_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Float(0.0))],
        // stack offset
        1,
    ))
}

fn math_cos_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let angle: f32 = vm.stack_peek(1)?;

    let c = angle.cos();

    Ok(Some(Var::Float(c)))
}

fn math_sin_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Float(0.0))],
        // stack offset
        1,
    ))
}

fn math_sin_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let angle: f32 = vm.stack_peek(1)?;

    let s = angle.sin();

    Ok(Some(Var::Float(s)))
}

fn prng_build_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Seed, Var::Float(1.0)),
            (Keyword::Min, Var::Float(0.0)),
            (Keyword::Max, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn prng_build_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let seed: f32 = vm.stack_peek(1)?;
    let min: f32 = vm.stack_peek(2)?;
    let max: f32 = vm.stack_peek(3)?;

    let prng_state_struct = prng::PrngStateStruct::new(seed as i32, min, max);

    Ok(Some(Var::PrngState(Rc::new(RefCell::new(
        prng_state_struct,
    )))))
}

fn prng_values_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::Num, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn prng_values_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("prng/values requires a from parameter");
        return Err(Error::Native);
    }

    let mut ref_mut_prng_state = ref_mut_prng_state_struct(&vm.stack, vm.sp, 1)?;
    let num: f32 = vm.stack_peek(2)?;
    let num = num as i32;

    let mut vs: Vec<Var> = Vec::new();
    for _ in 0..num {
        let f = ref_mut_prng_state.next_f32_defined_range();
        vs.push(Var::Float(f))
    }

    Ok(Some(Var::Vector(vs)))
}

fn prng_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Bool(false))],
        // stack offset
        1,
    ))
}

fn prng_value_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("prng/value requires a from parameter");
        return Err(Error::Native);
    }

    let mut ref_mut_prng_state = ref_mut_prng_state_struct(&vm.stack, vm.sp, 1)?;
    let res = ref_mut_prng_state.next_f32_defined_range();

    Ok(Some(Var::Float(res)))
}

fn prng_perlin_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::X, Var::Float(1.0)),
            (Keyword::Y, Var::Float(1.0)),
            (Keyword::Z, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn prng_perlin_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let x: f32 = vm.stack_peek(1)?;
    let y: f32 = vm.stack_peek(2)?;
    let z: f32 = vm.stack_peek(3)?;

    let res = prng::perlin(x, y, z);

    Ok(Some(Var::Float(res)))
}

fn interp_build_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::V2D(0.0, 1.0)),
            (Keyword::To, Var::V2D(0.0, 100.0)),
            (Keyword::Clamping, Var::Keyword(Keyword::False)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
        ],
        // stack offset
        1,
    ))
}

fn interp_build_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let from: (f32, f32) = vm.stack_peek(1)?;
    let to: (f32, f32) = vm.stack_peek(2)?;
    let clamping: Keyword = vm.stack_peek(3)?;
    let mapping: Keyword = vm.stack_peek(4)?;

    let clamping = clamping == Keyword::True;

    if let Some(mapping) = easing_from_keyword(mapping) {
        let from_m = mathutil::mc_m(from.0, 0.0, from.1, 1.0);
        let from_c = mathutil::mc_c(from.0, 0.0, from_m);
        let to_m = mathutil::mc_m(0.0, to.0, 1.0, to.1);
        let to_c = mathutil::mc_c(0.0, to.0, to_m);

        Ok(Some(Var::InterpState(interp::InterpStateStruct {
            from_m,
            to_m,
            from_c,
            to_c,
            to,
            clamping,
            mapping,
        })))
    } else {
        error!("interp_build_execute");
        Err(Error::Native)
    }
}

fn interp_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::T, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_value_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("interp/value requires a from parameter");
        return Err(Error::Native);
    }

    let interp_state = stack_peek_interp_state_struct(&vm.stack, vm.sp, 1)?;
    let t: f32 = vm.stack_peek(2)?;

    let res = interp_state.value(t);

    // error!("interp/value = {:?} t: {} interp_state: {:?}", res, t, &interp_state);

    Ok(Some(Var::Float(res)))
}

fn interp_cos_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Amplitude, Var::Float(1.0)),
            (Keyword::Frequency, Var::Float(1.0)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_cos_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let amplitude: f32 = vm.stack_peek(1)?;
    let frequency: f32 = vm.stack_peek(2)?;
    let t: f32 = vm.stack_peek(3)?;

    let res = interp::cos(amplitude, frequency, t);

    Ok(Some(Var::Float(res)))
}

fn interp_sin_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Amplitude, Var::Float(1.0)),
            (Keyword::Frequency, Var::Float(1.0)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_sin_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let amplitude: f32 = vm.stack_peek(1)?;
    let frequency: f32 = vm.stack_peek(2)?;
    let t: f32 = vm.stack_peek(3)?;

    let res = interp::sin(amplitude, frequency, t);

    Ok(Some(Var::Float(res)))
}

fn interp_bezier_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_bezier_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("interp/bezier requires coords parameter");
        return Err(Error::Native);
    }

    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let t: f32 = vm.stack_peek(2)?;

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        error!("coords 3 should be a Vec::V2D");
        return Err(Error::Native);
    };

    let (x, y) = interp::bezier(&[x0, y0, x1, y1, x2, y2, x3, y3], t);

    Ok(Some(Var::V2D(x, y)))
}

fn interp_bezier_tangent_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_bezier_tangent_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("interp/bezier requires coords parameter");
        return Err(Error::Native);
    }

    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let t: f32 = vm.stack_peek(2)?;

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        error!("coords 3 should be a Vec::V2D");
        return Err(Error::Native);
    };

    let (x, y) = interp::bezier_tangent(&[x0, y0, x1, y1, x2, y2, x3, y3], t);

    Ok(Some(Var::V2D(x, y)))
}

fn interp_ray_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Point, Var::V2D(0.0, 0.0)),
            (Keyword::Direction, Var::V2D(1000.0, 1000.0)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_ray_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let point: (f32, f32) = vm.stack_peek(1)?;
    let direction: (f32, f32) = vm.stack_peek(2)?;
    let t: f32 = vm.stack_peek(3)?;

    let (x, y) = interp::ray(point, direction, t);

    Ok(Some(Var::V2D(x, y)))
}

fn interp_line_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::V2D(0.0, 0.0)),
            (Keyword::To, Var::V2D(0.0, 0.0)),
            (Keyword::Clamping, Var::Keyword(Keyword::False)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
            (Keyword::T, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_line_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let from: (f32, f32) = vm.stack_peek(1)?;
    let to: (f32, f32) = vm.stack_peek(2)?;
    let clamping: Keyword = vm.stack_peek(3)?;
    let mapping: Keyword = vm.stack_peek(4)?;
    let t: f32 = vm.stack_peek(5)?;

    let clamping = clamping == Keyword::True;

    if let Some(mapping) = easing_from_keyword(mapping) {
        let x = interp::scalar(from.0, to.0, mapping, clamping, t);
        let y = interp::scalar(from.1, to.1, mapping, clamping, t);

        Ok(Some(Var::V2D(x, y)))
    } else {
        error!("interp_line_execute");
        Err(Error::Native)
    }
}

fn interp_circle_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Position, Var::V2D(0.0, 0.0)),
            (Keyword::Radius, Var::Float(1.0)),
            (Keyword::T, Var::Float(0.0)),
        ],
        // stack offset
        1,
    ))
}

fn interp_circle_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let position: (f32, f32) = vm.stack_peek(1)?;
    let radius: f32 = vm.stack_peek(2)?;
    let t: f32 = vm.stack_peek(3)?;

    let (x, y) = interp::circle(position, radius, t);

    Ok(Some(Var::V2D(x, y)))
}

fn path_linear_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::V2D(0.0, 0.0)),
            (Keyword::To, Var::V2D(10.0, 10.0)),
            (Keyword::Steps, Var::Float(10.0)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
        ],
        // stack offset
        0,
    ))
}

fn path_linear_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(8)?;

    if !is_arg_given(default_mask, 6) {
        error!("path_linear_execute requires fn argument");
        return Err(Error::Native);
    }

    let from: (f32, f32) = vm.stack_peek(1)?;
    let to: (f32, f32) = vm.stack_peek(2)?;
    let steps: f32 = vm.stack_peek(3)?;
    let t_start: f32 = vm.stack_peek(4)?;
    let t_end: f32 = vm.stack_peek(5)?;
    let fun: i32 = vm.stack_peek(6)?;
    let mapping: Keyword = vm.stack_peek(7)?;

    if let Some(mapping) = easing_from_keyword(mapping) {
        path::linear(
            vm,
            context,
            program,
            fun as usize,
            steps as i32,
            t_start,
            t_end,
            from.0,
            from.1,
            to.0,
            to.1,
            mapping,
        )?;
    }

    Ok(None)
}

fn path_circle_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Position, Var::V2D(0.0, 0.0)),
            (Keyword::Radius, Var::Float(100.0)),
            (Keyword::Steps, Var::Float(10.0)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
        ],
        // stack offset
        0,
    ))
}

fn path_circle_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(8)?;

    if !is_arg_given(default_mask, 6) {
        error!("path_circle_execute requires fn argument");
        return Err(Error::Native);
    }

    let position: (f32, f32) = vm.stack_peek(1)?;
    let radius: f32 = vm.stack_peek(2)?;
    let steps: f32 = vm.stack_peek(3)?;
    let t_start: f32 = vm.stack_peek(4)?;
    let t_end: f32 = vm.stack_peek(5)?;
    let fun: i32 = vm.stack_peek(6)?;
    let mapping: Keyword = vm.stack_peek(7)?;

    if let Some(mapping) = easing_from_keyword(mapping) {
        path::circular(
            vm,
            context,
            program,
            fun as usize,
            steps as i32,
            t_start,
            t_end,
            position.0,
            position.1,
            radius,
            mapping,
        )?;
    }

    Ok(None)
}

fn path_spline_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::Steps, Var::Float(10.0)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
        ],
        // stack offset
        0,
    ))
}

fn path_spline_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(7)?;

    if !is_arg_given(default_mask, 1) {
        error!("path_spline_execute requires coords argument");
        return Err(Error::Native);
    }
    if !is_arg_given(default_mask, 5) {
        error!("path_spline_execute requires fn argument");
        return Err(Error::Native);
    }

    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let steps: f32 = vm.stack_peek(2)?;
    let t_start: f32 = vm.stack_peek(3)?;
    let t_end: f32 = vm.stack_peek(4)?;
    let fun: i32 = vm.stack_peek(5)?;
    let mapping: Keyword = vm.stack_peek(6)?;

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };

    if let Some(mapping) = easing_from_keyword(mapping) {
        path::spline(
            vm,
            context,
            program,
            fun as usize,
            steps as i32,
            t_start,
            t_end,
            [x0, y0, x1, y1, x2, y2],
            mapping,
        )?;
    }

    Ok(None)
}

fn path_bezier_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Coords, Var::Bool(false)),
            (Keyword::Steps, Var::Float(10.0)),
            (Keyword::TStart, Var::Float(0.0)),
            (Keyword::TEnd, Var::Float(1.0)),
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
        ],
        // stack offset
        0,
    ))
}

fn path_bezier_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(7)?;

    if !is_arg_given(default_mask, 1) {
        error!("path_bezier_execute requires coords argument");
        return Err(Error::Native);
    }
    if !is_arg_given(default_mask, 5) {
        error!("path_bezier_execute requires fn argument");
        return Err(Error::Native);
    }

    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let steps: f32 = vm.stack_peek(2)?;
    let t_start: f32 = vm.stack_peek(3)?;
    let t_end: f32 = vm.stack_peek(4)?;
    let fun: i32 = vm.stack_peek(5)?;
    let mapping: Keyword = vm.stack_peek(6)?;

    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        error!("coords 0 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        error!("coords 1 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        error!("coords 2 should be a Vec::V2D");
        return Err(Error::Native);
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        error!("coords 3 should be a Vec::V2D");
        return Err(Error::Native);
    };

    if let Some(mapping) = easing_from_keyword(mapping) {
        path::bezier(
            vm,
            context,
            program,
            fun as usize,
            steps as i32,
            t_start,
            t_end,
            [x0, y0, x1, y1, x2, y2, x3, y3],
            mapping,
        )?;
    }

    Ok(None)
}

fn repeat_symmetry_vertical_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Fn, Var::Bool(false))],
        // stack offset
        0,
    ))
}

fn repeat_symmetry_vertical_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_symmetry_vertical requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;

    repeat::symmetry_vertical(vm, context, program, fun as usize)?;

    Ok(None)
}

fn repeat_symmetry_horizontal_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Fn, Var::Bool(false))],
        // stack offset
        0,
    ))
}

fn repeat_symmetry_horizontal_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_symmetry_horizontal requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;

    repeat::symmetry_horizontal(vm, context, program, fun as usize)?;

    Ok(None)
}

fn repeat_symmetry_4_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Fn, Var::Bool(false))],
        // stack offset
        0,
    ))
}

fn repeat_symmetry_4_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_symmetry_4 requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;

    repeat::symmetry_4(vm, context, program, fun as usize)?;

    Ok(None)
}

fn repeat_symmetry_8_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Fn, Var::Bool(false))],
        // stack offset
        0,
    ))
}

fn repeat_symmetry_8_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_symmetry_8 requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;

    repeat::symmetry_8(vm, context, program, fun as usize)?;

    Ok(None)
}

fn repeat_rotate_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Copies, Var::Float(3.0)),
        ],
        // stack offset
        0,
    ))
}

fn repeat_rotate_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_rotate requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;
    let copies: usize = vm.stack_peek(2)?;

    repeat::rotate(vm, context, program, fun as usize, copies)?;

    Ok(None)
}

fn repeat_rotate_mirrored_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::Copies, Var::Float(3.0)),
        ],
        // stack offset
        0,
    ))
}

fn repeat_rotate_mirrored_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("repeat_rotate_mirrored requires fn argument");
        return Err(Error::Native);
    }

    let fun: i32 = vm.stack_peek(1)?;
    let copies: usize = vm.stack_peek(2)?;

    repeat::rotate_mirrored(vm, context, program, fun as usize, copies)?;

    Ok(None)
}

fn focal_build_generic_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Mapping, Var::Keyword(Keyword::Linear)),
            (Keyword::Position, Var::V2D(0.0, 0.0)),
            (Keyword::Distance, Var::Float(1.0)),
            (Keyword::TransformPosition, Var::Keyword(Keyword::False)),
        ],
        // stack offset
        1,
    ))
}

fn focal_build_generic_execute(vm: &mut Vm, focal_type: focal::FocalType) -> Result<Option<Var>> {
    let mapping: Keyword = vm.stack_peek(1)?;
    let position: (f32, f32) = vm.stack_peek(2)?;
    let distance: f32 = vm.stack_peek(3)?;
    let transform_pos: Keyword = vm.stack_peek(4)?;

    let transform_pos = transform_pos == Keyword::True;

    if let Some(mapping) = easing_from_keyword(mapping) {
        Ok(Some(Var::FocalState(focal::FocalStateStruct {
            focal_type,
            mapping,
            position,
            distance,
            transform_pos,
        })))
    } else {
        error!("focal_build_generic _execute");
        Err(Error::Native)
    }
}

fn focal_build_point_execute(vm: &mut Vm) -> Result<Option<Var>> {
    focal_build_generic_execute(vm, focal::FocalType::Point)
}

fn focal_build_vline_execute(vm: &mut Vm) -> Result<Option<Var>> {
    focal_build_generic_execute(vm, focal::FocalType::VLine)
}

fn focal_build_hline_execute(vm: &mut Vm) -> Result<Option<Var>> {
    focal_build_generic_execute(vm, focal::FocalType::HLine)
}

fn focal_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::Position, Var::V2D(0.0, 0.0)),
        ],
        // stack offset
        1,
    ))
}

fn focal_value_execute(vm: &mut Vm, context: &mut Context) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("focal/value requires a from parameter");
        return Err(Error::Native);
    }

    let focal_state_struct = stack_peek_focal_state_struct(&vm.stack, vm.sp, 1)?;
    let position: (f32, f32) = vm.stack_peek(2)?;

    let res = focal_state_struct.value(context, position);

    // error!("focal/value: {} position: {:?} focal_state_struct: {:?}", res, &position, &focal_state_struct);

    Ok(Some(Var::Float(res)))
}

fn bitmap_each_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::Position, Var::V2D(500.0, 500.0)),
            (Keyword::Width, Var::Float(1000.0)),
            (Keyword::Height, Var::Float(1000.0)),
            (Keyword::Fn, Var::Bool(false)),
            (Keyword::ShuffleSeed, Var::Float(0.0)),
        ],
        // stack offset
        0,
    ))
}

fn bitmap_each_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(7)?;

    if !is_arg_given(default_mask, 1) {
        error!("bitmap/each requires a from parameter");
        return Err(Error::Native);
    }
    if !is_arg_given(default_mask, 5) {
        error!("bitmap/each requires a fn parameter");
        return Err(Error::Native);
    }

    let from: Iname = vm.stack_peek(1)?;
    let position: (f32, f32) = vm.stack_peek(2)?;
    let width: f32 = vm.stack_peek(3)?;
    let height: f32 = vm.stack_peek(4)?;
    let fun: i32 = vm.stack_peek(5)?;

    let shuffle_seed: Option<f32> = if is_arg_given(default_mask, 6) {
        let seed: f32 = vm.stack_peek(6)?;
        Some(seed)
    } else {
        None
    };

    bitmap::each(
        vm,
        context,
        program,
        fun as usize,
        from,
        position,
        width,
        height,
        shuffle_seed,
    )?;

    Ok(None)
}

fn bitmap_value_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::Position, Var::V2D(0.0, 0.0)),
            (Keyword::DefaultColour, Var::Colour(Colour::clear_colour())),
        ],
        // stack offset
        1,
    ))
}

fn bitmap_value_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(4)?;

    if !is_arg_given(default_mask, 1) {
        error!("bitmap/value requires a from parameter");
        return Err(Error::Native);
    }

    let from: Iname = vm.stack_peek(1)?;
    let position: (f32, f32) = vm.stack_peek(2)?;
    let default_colour: Colour = vm.stack_peek(3)?;

    let col = bitmap::value(context, program, from, position, default_colour)?;

    Ok(Some(Var::Colour(col)))
}

fn bitmap_width_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Bool(false))],
        // stack offset
        1,
    ))
}

fn bitmap_width_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("bitmap/width requires a from parameter");
        return Err(Error::Native);
    }

    let from: Iname = vm.stack_peek(1)?;

    let width = bitmap::width(context, program, from)?;

    Ok(Some(Var::Float(width)))
}

fn bitmap_height_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Bool(false))],
        // stack offset
        1,
    ))
}

fn bitmap_height_execute(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("bitmap/height requires a from parameter");
        return Err(Error::Native);
    }

    let from: Iname = vm.stack_peek(1)?;

    let height = bitmap::height(context, program, from)?;

    Ok(Some(Var::Float(height)))
}

fn mask_set_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::Invert, Var::Float(0.0)),
        ],
        // stack offset
        0,
    ))
}

fn mask_set_execute(vm: &mut Vm, context: &mut Context, program: &Program) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("mask/set requires a from parameter");
        return Err(Error::Native);
    }

    let from: Iname = vm.stack_peek(1)?;
    let invert_f32: f32 = vm.stack_peek(2)?; // hacky: should just work with a boolean

    let invert: bool = invert_f32 > 0.0;

    let mask_filename = program.data.string_from_iname(from)?;
    context.set_mask(&mask_filename, invert)?;

    Ok(None)
}

fn gen_stray_int_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Float(1.0)),
            (Keyword::By, Var::Float(0.2)),
        ],
        // stack offset
        1,
    ))
}

fn gen_stray_int_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let from: f32 = vm.stack_peek(1)?;
    let by: f32 = vm.stack_peek(2)?;

    let by = mathutil::absf(by);
    let value = vm.prng_state.next_f32_range(from - by, from + by);
    let value = value.floor();

    Ok(Some(Var::Float(value)))
}

fn gen_stray_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Float(1.0)),
            (Keyword::By, Var::Float(0.2)),
        ],
        // stack offset
        1,
    ))
}

fn gen_stray_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let from: f32 = vm.stack_peek(1)?;
    let by: f32 = vm.stack_peek(2)?;

    let by = mathutil::absf(by);
    let value = vm.prng_state.next_f32_range(from - by, from + by);

    Ok(Some(Var::Float(value)))
}

fn gen_stray_2d_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::V2D(10.0, 10.0)),
            (Keyword::By, Var::V2D(1.0, 1.0)),
        ],
        // stack offset
        1,
    ))
}

fn gen_stray_2d_execute(vm: &mut Vm) -> Result<Option<Var>> {
    if !vm.building_with_trait_within_vector {
        error!("gen_stray_2d should always be called with vm.building_with_trait_within_vector");
        return Err(Error::Native);
    }

    let from: (f32, f32) = vm.stack_peek(1)?;
    let by: (f32, f32) = vm.stack_peek(2)?;

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
        error!("gen_stray_2d invalid trait_within_vector_index value");
        return Err(Error::Native);
    }

    // pick a scalar between min and max
    let value = vm
        .prng_state
        .next_f32_range(from_index - by_index, from_index + by_index);

    Ok(Some(Var::Float(value)))
}

fn gen_stray_3d_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::By, Var::Bool(false)),
        ],
        // stack offset
        1,
    ))
}

fn gen_stray_3d_execute(vm: &mut Vm) -> Result<Option<Var>> {
    if !vm.building_with_trait_within_vector {
        error!("gen_stray_3d should always be called with vm.building_with_trait_within_vector");
        return Err(Error::Native);
    }

    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("gen/stray-3d requires a from parameter");
        return Err(Error::Native);
    }
    if !is_arg_given(default_mask, 2) {
        error!("gen/stray-3d requires a by parameter");
        return Err(Error::Native);
    }

    let from = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let by = stack_peek_vars(&vm.stack, vm.sp, 2)?;

    let index = vm.trait_within_vector_index;

    let from = if let Some(var) = from.get(index) {
        Var::get_float_value(&var)?
    } else {
        error!("gen_stray_3d requires both from and by parameters");
        return Err(Error::Native);
    };

    let by = if let Some(var) = by.get(index) {
        Var::get_float_value(&var)?
    } else {
        error!("gen_stray_3d requires both from and by parameters");
        return Err(Error::Native);
    };

    // pick a scalar between min and max
    let value = vm.prng_state.next_f32_range(from - by, from + by);

    Ok(Some(Var::Float(value)))
}

fn gen_stray_4d_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::From, Var::Bool(false)),
            (Keyword::By, Var::Bool(false)),
        ],
        // stack offset
        1,
    ))
}

fn gen_stray_4d_execute(vm: &mut Vm) -> Result<Option<Var>> {
    if !vm.building_with_trait_within_vector {
        error!("gen_stray_4d should always be called with vm.building_with_trait_within_vector");
        return Err(Error::Native);
    }

    let default_mask: i32 = vm.stack_peek(3)?;

    if !is_arg_given(default_mask, 1) {
        error!("gen/stray-4d requires a from parameter");
        return Err(Error::Native);
    }
    if !is_arg_given(default_mask, 2) {
        error!("gen/stray-4d requires a by parameter");
        return Err(Error::Native);
    }

    let from = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let by = stack_peek_vars(&vm.stack, vm.sp, 2)?;

    let index = vm.trait_within_vector_index;

    let from = if let Some(var) = from.get(index) {
        Var::get_float_value(&var)?
    } else {
        error!("gen_stray_4d requires both from and by parameters");
        return Err(Error::Native);
    };

    let by = if let Some(var) = by.get(index) {
        Var::get_float_value(&var)?
    } else {
        error!("gen_stray_4d requires both from and by parameters");
        return Err(Error::Native);
    };

    // pick a scalar between min and max
    let value = vm.prng_state.next_f32_range(from - by, from + by);

    Ok(Some(Var::Float(value)))
}

fn gen_int_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Min, Var::Float(0.0)),
            (Keyword::Max, Var::Float(1000.0)),
        ],
        // stack offset
        1,
    ))
}

fn gen_int_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let min: f32 = vm.stack_peek(1)?;
    let max: f32 = vm.stack_peek(2)?;

    // pick a scalar between min and max
    let value = vm.prng_state.next_f32_range(min, max + 1.0);

    Ok(Some(Var::Float(value.floor())))
}

fn gen_scalar_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Min, Var::Float(0.0)),
            (Keyword::Max, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn gen_scalar_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let min: f32 = vm.stack_peek(1)?;
    let max: f32 = vm.stack_peek(2)?;

    // pick a scalar between min and max
    let value = vm.prng_state.next_f32_range(min, max);

    Ok(Some(Var::Float(value)))
}

fn gen_2d_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Min, Var::Float(0.0)),
            (Keyword::Max, Var::Float(1.0)),
        ],
        // stack offset
        1,
    ))
}

fn gen_2d_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let min: f32 = vm.stack_peek(1)?;
    let max: f32 = vm.stack_peek(2)?;

    let x = vm.prng_state.next_f32_range(min, max);
    let y = vm.prng_state.next_f32_range(min, max);

    Ok(Some(Var::V2D(x, y)))
}

fn gen_select_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::From, Var::Bool(false))],
        // stack offset
        1,
    ))
}

fn gen_select_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    if !is_arg_given(default_mask, 1) {
        error!("gen/select requires a from parameter");
        return Err(Error::Native);
    }

    let from = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let index = vm.prng_state.next_usize_range(0, from.len());

    Ok(Some(from[index].clone()))
}

fn gen_col_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![(Keyword::Alpha, Var::Float(1.0))],
        // stack offset
        1,
    ))
}

fn gen_col_execute(vm: &mut Vm) -> Result<Option<Var>> {
    let default_mask: i32 = vm.stack_peek(2)?;

    let alpha: f32 = if is_arg_given(default_mask, 1) {
        vm.stack_peek(1)?
    } else {
        // no alpha was given so generate a random value
        vm.prng_state.next_f32_range(0.0, 1.0)
    };

    Ok(Some(Var::Colour(Colour::new(
        ColourFormat::Rgb,
        vm.prng_state.next_f32_range(0.0, 1.0),
        vm.prng_state.next_f32_range(0.0, 1.0),
        vm.prng_state.next_f32_range(0.0, 1.0),
        alpha,
    ))))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::colour::ColourFormat;
    // use crate::geometry::RENDER_PACKET_FLOAT_PER_VERTEX;
    use crate::vm::tests::*;
    use crate::vm::*;

    fn is_col_rgb(s: &str, r: f32, g: f32, b: f32, alpha: f32) {
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();
        if let Var::Colour(col) = vm_exec(&mut vm, &mut context, s) {
            assert_eq!(col.format, ColourFormat::Rgb);
            assert_eq!(col.e0, r);
            assert_eq!(col.e1, g);
            assert_eq!(col.e2, b);
            assert_eq!(col.e3, alpha);
        }
    }
    /*
        // get render packet 0's geometry length
        fn rp0_num_vertices(context: &Context, expected_num_vertices: usize) {
            assert_eq!(
                context.get_render_packet_geo_len(0),
                expected_num_vertices * RENDER_PACKET_FLOAT_PER_VERTEX
            );
        }

        // #[test]
        fn dev_rendering_fns() {
            let mut vm: Vm = Default::default();
            let mut context: Context = Default::default();
            vm_run(&mut vm, &mut context, "(line width: 33 from: [2 3] to: [400 500] colour: (col/rgb r: 0 g: 0.1 b: 0.2 alpha: 0.3))");
            // vm_run(&mut vm, "(line width: 0 from: [2 3] to: [400 500] brush: brush/b colour: (col/rgb r: 0 g: 0.1 b: 0.2 alpha: 0.3))");
            // vm_run(&mut vm, "(line brush: brush/b)");
            // vm_run(&mut vm, "(line brush: brush/b) (rect width: 10 height: 30)");

            let res = vm.top_stack_value().unwrap();
            if let Var::Debug(s) = res {
                assert_eq!(s, "x");
            } else {
                assert_eq!(false, true);
            }

            rp0_num_vertices(&context, 4);
        }
    */
    #[test]
    fn test_native_pack() {
        let mut res: String = "".to_string();
        Native::ColAlpha.pack(&mut res).unwrap();
        assert_eq!("col/alpha", res);
    }

    #[test]
    fn test_native_unpack() {
        let (res, _rem) = Native::unpack("col/alpha").unwrap();
        assert_eq!(res, Native::ColAlpha);
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
    pub fn bug_nth() {
        is_debug_str(
            "(define vs [5 6 7])
             (probe scalar: (nth from: vs n: 0))",
            "5",
        );
    }

    #[test]
    fn test_vector_length() {
        is_int("(define v []) (++ v 100) (vector/length from: v)", 1);
        is_int("(define v [1]) (++ v 100) (vector/length from: v)", 2);
        is_int("(define v [1 2]) (++ v 100) (vector/length from: v)", 3);
        is_int("(define v [1 2 3]) (++ v 100) (vector/length from: v)", 4);
        is_int("(define v [1 2 3 4]) (++ v 100) (vector/length from: v)", 5);
        is_int(
            "(define v []) (++ v 4) (++ v 3) (++ v 2) (++ v 1) (++ v 0) (vector/length from: v)",
            5,
        );
        is_int(
            "(define v [1 2]) (++ v 98) (++ v 99) (++ v 100) (vector/length from: v)",
            5,
        );
    }

    #[test]
    fn test_math() {
        is_float("(math/clamp from: 3 min: 2 max: 5)", 3.0);
        is_float("(math/clamp from: 1 min: 2 max: 5)", 2.0);
        is_float("(math/clamp from: 8 min: 2 max: 5)", 5.0);

        is_float("(math/radians->degrees from: 0.3)", 17.188734);

        is_float("(math/cos from: 0.7)", 0.7648422);
        is_float("(math/sin from: 0.9)", 0.7833269);
    }
    #[test]
    fn dev_new_args() {
        is_float("(math/clamp from: 3 min: 2 max: 5)", 3.0);
        is_float("(math/clamp from: 1 min: 2 max: 5)", 2.0);
        is_float("(math/clamp from: 8 min: 2 max: 5)", 5.0);
    }

}
