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

use crate::ease::easing_from_keyword;
use crate::builtin::Builtin;
use crate::colour::ColourFormat;
use crate::error::{Error, Result};
use crate::keywords::Keyword;
use crate::packable::{Mule, Packable};
use crate::vm::{Var, Vm};

use crate::uvmapper::BrushType;

use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
// use log::error;

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

fn is_arg_given(bits: i32, position: usize) -> bool {
    // note: the position value will corresspond to the stack_peek value.
    // stack peek values start at 1 rather than 0 (since the current stack
    // pointer points to the 'next' free stack location). therefore we'll
    // need to subtract one from the position value
    (bits & (1 << (position - 1))) == 0
}

// can't have this as a member of Vm thanks to the borrow checker
fn stack_peek_vars(stack: &Vec<Var>, sp: usize, offset: usize) -> Result<&Vec<Var>> {
    if let Var::Vector(vs) = &stack[sp - offset] {
        Ok(vs)
    } else {
        return Err(Error::VM("expected Var::Vector".to_string()));
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

// return a tuple
// .0 == input arguments as a vector of (name, default value) pairs
// .1 == how the native function affects the vm's stack in terms of opcode offset
//
pub fn parameter_info(native: &Native) -> Result<(Vec<(Keyword, Var)>, i32)> {
    match native {
        Native::Line => line_parameter_info(),
        Native::Rect => rect_parameter_info(),
        Native::Circle => circle_parameter_info(),
        Native::CircleSlice => circle_slice_parameter_info(),
        Native::Poly => poly_parameter_info(),
        Native::Quadratic => quadratic_parameter_info(),
        Native::Bezier => bezier_parameter_info(),
        Native::BezierBulging => bezier_bulging_parameter_info(),
        _ => Err(Error::Native("parameter_info".to_string())),
    }
}

pub fn execute_native(vm: &mut Vm, native: &Native) -> Result<()> {
    match native {
        Native::Line => line_execute(vm),
        Native::Rect => rect_execute(vm),
        Native::Circle => circle_execute(vm),
        Native::CircleSlice => circle_slice_execute(vm),
        Native::Poly => poly_execute(vm),
        Native::Quadratic => quadratic_execute(vm),
        Native::Bezier => bezier_execute(vm),
        Native::BezierBulging => bezier_bulging_execute(vm),
        _ => Err(Error::Native("execute_native".to_string())),
    }
}

// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------
// --------------------------------------------------------------------------------

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

fn line_execute(vm: &mut Vm) -> Result<()> {
    let line_width = vm.stack_peek_f32(1)?;
    let line_from = vm.stack_peek_v2d(2)?;
    let line_to = vm.stack_peek_v2d(3)?;
    let from_col = vm.stack_peek_col(4)?;
    let to_col = vm.stack_peek_col(5)?;
    let col = vm.stack_peek_col(6)?;
    let brush = vm.stack_peek_kw(7)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(8)?;

    let default_mask = vm.stack_peek_i32(9)?;

    let brush_type = read_brush(brush);

    // if the from-colour and to-colour parameters are given
    if is_arg_given(default_mask, 4) && is_arg_given(default_mask, 5) {
        vm.render_line(line_from, line_to, line_width, &from_col, &to_col, brush_type, brush_subtype)
    } else {
        vm.render_line(line_from, line_to, line_width, &col, &col, brush_type, brush_subtype)
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

fn circle_execute(vm: &mut Vm) -> Result<()> {
    let width = vm.stack_peek_f32(1)?;
    let height = vm.stack_peek_f32(2)?;
    let position = vm.stack_peek_v2d(3)?;
    let col = vm.stack_peek_col(4)?;
    let tessellation = vm.stack_peek_f32_as_usize(5)?;
    let radius = vm.stack_peek_f32(6)?;

    let default_mask = vm.stack_peek_i32(7)?;

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {
        if is_arg_given(default_mask, 6) {
            // given a radius value
            vm.render_circle(position, radius, radius, &rgb, tessellation)
        } else {
            // radius was not explicitly specified
            vm.render_circle(position, width, height, &rgb, tessellation)
        }
    } else {
        Err(Error::Native("circle".to_string()))
    }
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

fn circle_slice_execute(vm: &mut Vm) -> Result<()> {
    let width = vm.stack_peek_f32(1)?;
    let height = vm.stack_peek_f32(2)?;
    let position = vm.stack_peek_v2d(3)?;
    let col = vm.stack_peek_col(4)?;
    let tessellation = vm.stack_peek_f32_as_usize(5)?;
    let radius = vm.stack_peek_f32(6)?;
    let angle_start = vm.stack_peek_f32(7)?;
    let angle_end = vm.stack_peek_f32(8)?;
    let inner_width = vm.stack_peek_f32(9)?;
    let inner_height = vm.stack_peek_f32(10)?;

    let default_mask = vm.stack_peek_i32(11)?;

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {
        if is_arg_given(default_mask, 6) {
            // given a radius value
            vm.render_circle_slice(
                position,
                radius,
                radius,
                &rgb,
                tessellation,
                angle_start,
                angle_end,
                inner_width,
                inner_height,
            )
        } else {
            // radius was not explicitly specified
            vm.render_circle_slice(
                position,
                width,
                height,
                &rgb,
                tessellation,
                angle_start,
                angle_end,
                inner_width,
                inner_height,
            )
        }
    } else {
        Err(Error::Native("circle_slice".to_string()))
    }
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

fn poly_execute(vm: &mut Vm) -> Result<()> {
    let default_mask = vm.stack_peek_i32(3)?;

    if !is_arg_given(default_mask, 1) || !is_arg_given(default_mask, 2) {
        return Err(Error::Native(
            "poly requires both coords and colours".to_string(),
        ));
    }

    // code looks like this thanks to the borrow checker being anal
    let coords = stack_peek_vars(&vm.stack, vm.sp, 1)?;
    let colours = stack_peek_vars(&vm.stack, vm.sp, 2)?;

    let geo = &mut vm.geometry;
    let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
        matrix
    } else {
        return Err(Error::Native("poly_execute: matrix required".to_string()));
    };
    let uv_mapping = vm.mappings.get_uv_mapping(BrushType::Flat, 0);

    geo.render_poly(matrix, coords, colours, uv_mapping)?;

    Ok(())
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

fn quadratic_execute(vm: &mut Vm) -> Result<()> {
    let line_width = vm.stack_peek_f32(1)?;
    let line_width_start = vm.stack_peek_f32(2)?;
    let line_width_end = vm.stack_peek_f32(3)?;
    let line_width_mapping = vm.stack_peek_kw(4)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 5)?;
    let t_start = vm.stack_peek_f32(6)?;
    let t_end = vm.stack_peek_f32(7)?;
    let tessellation = vm.stack_peek_f32_as_usize(8)?;
    let col = vm.stack_peek_col(9)?;
    let brush = vm.stack_peek_kw(10)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(11)?;

    let default_mask = vm.stack_peek_i32(12)?;

    if !is_arg_given(default_mask, 5) {
        return Err(Error::Native("quadratic requires coords".to_string()));
    }

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {

        let geo = &mut vm.geometry;
        let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
            matrix
        } else {
            return Err(Error::Native("quadratic: matrix required".to_string()));
        };
        let brush_type = read_brush(brush);
        let uv_mapping = vm.mappings.get_uv_mapping(brush_type, brush_subtype);

        if let Some(mapping) = easing_from_keyword(line_width_mapping) {
            if is_arg_given(default_mask, 1) {
                // given a line width value
                geo.render_quadratic_vars(
                    matrix,
                    coords,
                    line_width,
                    line_width,
                    mapping,
                    t_start,
                    t_end,
                    &rgb,
                    tessellation,
                    uv_mapping,
                )
            } else {
                // not given a line width value
                geo.render_quadratic_vars(
                    matrix,
                    coords,
                    line_width_start,
                    line_width_end,
                    mapping,
                    t_start,
                    t_end,
                    &rgb,
                    tessellation,
                    uv_mapping,
                )
            }
        } else {
            Err(Error::Native("quadratic: invalid mapping".to_string()))
        }
    } else {
        Err(Error::Native("quadratic: colour conversion".to_string()))
    }
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

fn bezier_execute(vm: &mut Vm) -> Result<()> {
    let line_width = vm.stack_peek_f32(1)?;
    let line_width_start = vm.stack_peek_f32(2)?;
    let line_width_end = vm.stack_peek_f32(3)?;
    let line_width_mapping = vm.stack_peek_kw(4)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 5)?;
    let t_start = vm.stack_peek_f32(6)?;
    let t_end = vm.stack_peek_f32(7)?;
    let tessellation = vm.stack_peek_f32_as_usize(8)?;
    let col = vm.stack_peek_col(9)?;
    let brush = vm.stack_peek_kw(10)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(11)?;

    let default_mask = vm.stack_peek_i32(12)?;

    if !is_arg_given(default_mask, 5) {
        return Err(Error::Native("bezier requires coords".to_string()));
    }

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {

        let geo = &mut vm.geometry;
        let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
            matrix
        } else {
            return Err(Error::Native("bezier: matrix required".to_string()));
        };
        let brush_type = read_brush(brush);
        let uv_mapping = vm.mappings.get_uv_mapping(brush_type, brush_subtype);

        if let Some(mapping) = easing_from_keyword(line_width_mapping) {
            if is_arg_given(default_mask, 1) {
                // given a line width value
                geo.render_bezier_vars(
                    matrix,
                    coords,
                    line_width,
                    line_width,
                    mapping,
                    t_start,
                    t_end,
                    &rgb,
                    tessellation,
                    uv_mapping,
                )
            } else {
                // not given a line width value
                geo.render_bezier_vars(
                    matrix,
                    coords,
                    line_width_start,
                    line_width_end,
                    mapping,
                    t_start,
                    t_end,
                    &rgb,
                    tessellation,
                    uv_mapping,
                )
            }
        } else {
            Err(Error::Native("bezier: invalid mapping".to_string()))
        }
    } else {
        Err(Error::Native("bezier: colour conversion".to_string()))
    }
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

fn bezier_bulging_execute(vm: &mut Vm) -> Result<()> {
    let line_width = vm.stack_peek_f32(1)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 2)?;
    let t_start = vm.stack_peek_f32(3)?;
    let t_end = vm.stack_peek_f32(4)?;
    let tessellation = vm.stack_peek_f32_as_usize(5)?;
    let col = vm.stack_peek_col(6)?;
    let brush = vm.stack_peek_kw(7)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(8)?;

    let default_mask = vm.stack_peek_i32(9)?;

    if !is_arg_given(default_mask, 2) {
        return Err(Error::Native("bezier_bulging requires coords".to_string()));
    }

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {

        let geo = &mut vm.geometry;
        let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
            matrix
        } else {
            return Err(Error::Native("bezier_bulging: matrix required".to_string()));
        };
        let brush_type = read_brush(brush);
        let uv_mapping = vm.mappings.get_uv_mapping(brush_type, brush_subtype);
        geo.render_bezier_bulging_vars(
            matrix,
            coords,
            line_width,
            t_start,
            t_end,
            &rgb,
            tessellation,
            uv_mapping)
    } else {
        Err(Error::Native("bezier_bulging: colour conversion".to_string()))
    }
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

fn stroked_bezier_execute(vm: &mut Vm) -> Result<()> {
    let tessellation = vm.stack_peek_f32_as_usize(1)?;
    let coords = stack_peek_vars(&vm.stack, vm.sp, 2)?;
    let stroke_tessellation = vm.stack_peek_f32_as_usize(3)?;
    let stroke_noise = vm.stack_peek_f32(4)?;
    let stroke_line_width_start = vm.stack_peek_f32(5)?;
    let stroke_line_width_end = vm.stack_peek_f32(6)?;
    let col = vm.stack_peek_col(7)?;
    let col_volatility = vm.stack_peek_f32(8)?;
    let seed = vm.stack_peek_f32(9)?;
    let line_width_mapping = vm.stack_peek_kw(10)?;
    let brush = vm.stack_peek_kw(11)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(12)?;

    let default_mask = vm.stack_peek_i32(13)?;

    if !is_arg_given(default_mask, 2) {
        return Err(Error::Native("stroked bezier requires coords".to_string()));
    }

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {

        let geo = &mut vm.geometry;
        let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
            matrix
        } else {
            return Err(Error::Native("stroked bezier: matrix required".to_string()));
        };
        let brush_type = read_brush(brush);
        let uv_mapping = vm.mappings.get_uv_mapping(brush_type, brush_subtype);

        if let Some(mapping) = easing_from_keyword(line_width_mapping) {
            geo.render_stroked_bezier_vars(
                matrix,
                tessellation,
                coords,
                stroke_tessellation,
                stroke_noise,
                stroke_line_width_start,
                stroke_line_width_end,
                &rgb,
                col_volatility,
                seed,
                mapping,
                uv_mapping,
            )
        } else {
            Err(Error::Native("stroked bezier: invalid mapping".to_string()))
        }
    } else {
        Err(Error::Native("stroked bezier: colour conversion".to_string()))
    }
}

fn stroked_bezier_rect_parameter_info() -> Result<(Vec<(Keyword, Var)>, i32)> {
    Ok((
        // input arguments
        vec![
            (Keyword::Position, Var::V2D(100.0, 100.0)),
            (Keyword::Width, Var::Float(80.0)),
            (Keyword::Height, Var::Float(600.0)),
            (Keyword::Volatility, Var::Float(30.0)),
            (Keyword::Overlap, Var::Float(0.0)),
            (Keyword::Iterations, Var::Float(10.0)),
            (Keyword::Seed, Var::Float(0.0)),
            (Keyword::Tessellation, Var::Float(15.0)),
            (Keyword::StrokeTessellation, Var::Float(10.0)),
            (Keyword::StrokeNoise, Var::Float(25.0)),
            (Keyword::Colour, Var::Colour(Default::default())),
            (Keyword::ColourVolatility, Var::Float(0.0)),
            (Keyword::Brush, Var::Keyword(Keyword::BrushFlat)),
            (Keyword::BrushSubtype, Var::Float(1.0)),
        ],
        // stack offset
        0,
    ))
}

fn stroked_bezier_rect_execute(vm: &mut Vm) -> Result<()> {
    let position = vm.stack_peek_v2d(1)?;
    let width = vm.stack_peek_f32(2)?;
    let height = vm.stack_peek_f32(3)?;
    let volatility = vm.stack_peek_f32(4)?;
    let overlap = vm.stack_peek_f32(5)?;
    let iterations = vm.stack_peek_f32(6)?;
    let seed = vm.stack_peek_f32(7)?;
    let tessellation = vm.stack_peek_f32_as_usize(8)?;
    let stroke_tessellation = vm.stack_peek_f32_as_usize(9)?;
    let stroke_noise = vm.stack_peek_f32(10)?;
    let col = vm.stack_peek_col(11)?;
    let col_volatility = vm.stack_peek_f32(12)?;
    let brush = vm.stack_peek_kw(13)?;
    let brush_subtype = vm.stack_peek_f32_as_usize(14)?;

    if let Ok(rgb) = col.convert(ColourFormat::Rgb) {
        let geo = &mut vm.geometry;
        let matrix = if let Some(matrix) = vm.matrix_stack.peek() {
            matrix
        } else {
            return Err(Error::Native("stroked bezier rect: matrix required".to_string()));
        };
        let brush_type = read_brush(brush);
        let uv_mapping = vm.mappings.get_uv_mapping(brush_type, brush_subtype);

        geo.render_stroked_bezier_rect(
            matrix,
            position,
            width,
            height,
            volatility,
            overlap,
            iterations,
            seed as i32,
            tessellation,
            stroke_tessellation,
            stroke_noise,
            &rgb,
            col_volatility,
            uv_mapping,
        )
    } else {
        Err(Error::Native("stroked bezier rect: colour conversion".to_string()))
    }
}
