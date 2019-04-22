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

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub type BuiltinCallback = fn(&mut Vm, &Program, usize) -> Result<Var>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString, EnumIter)]
pub enum Builtin {
    #[strum(serialize = "UnreachableBuiltinStart")]
    BuiltinStart = Keyword::KeywordEnd as isize,

    // misc
    //
    #[strum(serialize = "debug/print")]
    DebugPrint,
    #[strum(serialize = "BUILTIN-nth")]
    Nth,
    #[strum(serialize = "BUILTIN-vector/length")]
    VectorLength,
    #[strum(serialize = "BUILTIN-probe")]
    Probe,

    // shapes
    //
    #[strum(serialize = "BUILTIN-line")]
    Line,
    #[strum(serialize = "BUILTIN-rect")]
    Rect,
    #[strum(serialize = "BUILTIN-circle")]
    Circle,
    #[strum(serialize = "BUILTIN-circle-slice")]
    CircleSlice,
    #[strum(serialize = "BUILTIN-poly")]
    Poly,
    #[strum(serialize = "BUILTIN-quadratic")]
    Quadratic,
    #[strum(serialize = "BUILTIN-bezier")]
    Bezier,
    #[strum(serialize = "BUILTIN-bezier-bulging")]
    BezierBulging,
    #[strum(serialize = "BUILTIN-stroked-bezier")]
    StrokedBezier,
    #[strum(serialize = "BUILTIN-stroked-bezier-rect")]
    StrokedBezierRect,

    // transforms
    //
    #[strum(serialize = "BUILTIN-translate")]
    Translate,
    #[strum(serialize = "BUILTIN-rotate")]
    Rotate,
    #[strum(serialize = "BUILTIN-scale")]
    Scale,

    // colour
    //
    #[strum(serialize = "BUILTIN-col/convert")]
    ColConvert,
    #[strum(serialize = "BUILTIN-col/rgb")]
    ColRGB,
    #[strum(serialize = "BUILTIN-col/hsl")]
    ColHSL,
    #[strum(serialize = "BUILTIN-col/hsluv")]
    ColHSLuv,
    #[strum(serialize = "BUILTIN-col/hsv")]
    ColHSV,
    #[strum(serialize = "BUILTIN-col/lab")]
    ColLAB,
    #[strum(serialize = "BUILTIN-col/complementary")]
    ColComplementary,
    #[strum(serialize = "BUILTIN-col/split-complementary")]
    ColSplitComplementary,
    #[strum(serialize = "BUILTIN-col/analagous")]
    ColAnalagous,
    #[strum(serialize = "BUILTIN-col/triad")]
    ColTriad,
    #[strum(serialize = "BUILTIN-col/darken")]
    ColDarken,
    #[strum(serialize = "BUILTIN-col/lighten")]
    ColLighten,
    #[strum(serialize = "BUILTIN-col/set-alpha")]
    ColSetAlpha,
    #[strum(serialize = "BUILTIN-col/get-alpha")]
    ColGetAlpha,
    #[strum(serialize = "BUILTIN-col/set-r")]
    ColSetR,
    #[strum(serialize = "BUILTIN-col/get-r")]
    ColGetR,
    #[strum(serialize = "BUILTIN-col/set-g")]
    ColSetG,
    #[strum(serialize = "BUILTIN-col/get-g")]
    ColGetG,
    #[strum(serialize = "BUILTIN-col/set-b")]
    ColSetB,
    #[strum(serialize = "BUILTIN-col/get-b")]
    ColGetB,
    #[strum(serialize = "BUILTIN-col/set-h")]
    ColSetH,
    #[strum(serialize = "BUILTIN-col/get-h")]
    ColGetH,
    #[strum(serialize = "BUILTIN-col/set-s")]
    ColSetS,
    #[strum(serialize = "BUILTIN-col/get-s")]
    ColGetS,
    #[strum(serialize = "BUILTIN-col/set-l")]
    ColSetL,
    #[strum(serialize = "BUILTIN-col/get-l")]
    ColGetL,
    #[strum(serialize = "BUILTIN-col/set-a")]
    ColSetA,
    #[strum(serialize = "BUILTIN-col/get-a")]
    ColGetA,
    #[strum(serialize = "BUILTIN-col/set-v")]
    ColSetV,
    #[strum(serialize = "BUILTIN-col/get-v")]
    ColGetV,
    #[strum(serialize = "BUILTIN-col/build-procedural")]
    ColBuildProcedural,
    #[strum(serialize = "col/build-bezier")]
    ColBuildBezier,
    #[strum(serialize = "BUILTIN-col/value")]
    ColValue,

    // math
    //
    #[strum(serialize = "BUILTIN-math/distance")]
    MathDistance,
    #[strum(serialize = "BUILTIN-math/normal")]
    MathNormal,
    #[strum(serialize = "BUILTIN-math/clamp")]
    MathClamp,
    #[strum(serialize = "BUILTIN-math/radians->degrees")]
    MathRadiansDegrees,
    #[strum(serialize = "BUILTIN-math/cos")]
    MathCos,
    #[strum(serialize = "BUILTIN-math/sin")]
    MathSin,

    // prng
    //
    #[strum(serialize = "BUILTIN-prng/build")]
    PrngBuild,
    #[strum(serialize = "BUILTIN-prng/values")]
    PrngValues,
    #[strum(serialize = "BUILTIN-prng/value")]
    PrngValue,
    #[strum(serialize = "BUILTIN-prng/perlin")]
    PrngPerlin,

    // interp
    //
    #[strum(serialize = "BUILTIN-interp/build")]
    InterpBuild,
    #[strum(serialize = "BUILTIN-interp/value")]
    InterpValue,
    #[strum(serialize = "BUILTIN-interp/cos")]
    InterpCos,
    #[strum(serialize = "BUILTIN-interp/sin")]
    InterpSin,
    #[strum(serialize = "BUILTIN-interp/bezier")]
    InterpBezier,
    #[strum(serialize = "BUILTIN-interp/bezier-tangent")]
    InterpBezierTangent,
    #[strum(serialize = "BUILTIN-interp/ray")]
    InterpRay,
    #[strum(serialize = "BUILTIN-interp/line")]
    InterpLine,
    #[strum(serialize = "BUILTIN-interp/circle")]
    InterpCircle,

    // path
    //
    #[strum(serialize = "BUILTIN-path/linear")]
    PathLinear,
    #[strum(serialize = "BUILTIN-path/circle")]
    PathCircle,
    #[strum(serialize = "BUILTIN-path/spline")]
    PathSpline,
    #[strum(serialize = "BUILTIN-path/bezier")]
    PathBezier,

    // repeat
    //
    #[strum(serialize = "BUILTIN-repeat/symmetry-vertical")]
    RepeatSymmetryVertical,
    #[strum(serialize = "BUILTIN-repeat/symmetry-horizontal")]
    RepeatSymmetryHorizontal,
    #[strum(serialize = "BUILTIN-repeat/symmetry-4")]
    RepeatSymmetry4,
    #[strum(serialize = "BUILTIN-repeat/symmetry-8")]
    RepeatSymmetry8,
    #[strum(serialize = "BUILTIN-repeat/rotate")]
    RepeatRotate,
    #[strum(serialize = "BUILTIN-repeat/rotate-mirrored")]
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

    #[strum(serialize = "UnreachableBuiltinEnd")]
    BuiltinEnd,
}

impl Packable for Builtin {
    fn pack(&self, cursor: &mut String) -> Result<()> {
        Mule::pack_label(cursor, &self.to_string());

        Ok(())
    }

    fn unpack(cursor: &str) -> Result<(Self, &str)> {
        let ns = Mule::next_space(cursor);
        let sub = &cursor[0..ns];
        let res = sub.parse::<Builtin>()?;

        Ok((res, &cursor[ns..]))
    }
}

pub fn i32_to_builtin_hash() -> HashMap<i32, Builtin> {
    let mut hm: HashMap<i32, Builtin> = HashMap::new();

    for n in Builtin::iter() {
        hm.insert(n as i32, n);
    }

    hm
}

pub fn build_builtin_fn_hash() -> HashMap<Builtin, BuiltinCallback> {
    let mut h: HashMap<Builtin, BuiltinCallback> = HashMap::new();

    // --------------------------------------------------
    // misc
    // --------------------------------------------------
    // BIND("debug/print", debug_print);
    h.insert(Builtin::Nth, nth);
    h.insert(Builtin::VectorLength, vector_length);
    h.insert(Builtin::Probe, probe);
    // map (todo)

    // --------------------------------------------------
    // shapes
    // --------------------------------------------------
    h.insert(Builtin::Line, line);
    h.insert(Builtin::Rect, rect);
    h.insert(Builtin::Circle, circle);
    h.insert(Builtin::CircleSlice, circle_slice);
    h.insert(Builtin::Poly, poly);
    h.insert(Builtin::Quadratic, quadratic);
    h.insert(Builtin::Bezier, bezier);
    h.insert(Builtin::BezierBulging, bezier_bulging);
    h.insert(Builtin::StrokedBezier, stroked_bezier);
    h.insert(Builtin::StrokedBezierRect, stroked_bezier_rect);

    // --------------------------------------------------
    // transforms
    // --------------------------------------------------
    h.insert(Builtin::Translate, translate);
    h.insert(Builtin::Rotate, rotate);
    h.insert(Builtin::Scale, scale);

    // --------------------------------------------------
    // colour
    // --------------------------------------------------
    h.insert(Builtin::ColConvert, col_convert);
    h.insert(Builtin::ColRGB, col_rgb);
    h.insert(Builtin::ColHSL, col_hsl);
    h.insert(Builtin::ColHSLuv, col_hsluv);
    h.insert(Builtin::ColHSV, col_hsv);
    h.insert(Builtin::ColLAB, col_lab);
    h.insert(Builtin::ColComplementary, col_complementary);
    h.insert(Builtin::ColSplitComplementary, col_split_complementary);
    h.insert(Builtin::ColAnalagous, col_analagous);
    h.insert(Builtin::ColTriad, col_triad);
    h.insert(Builtin::ColDarken, col_darken);
    h.insert(Builtin::ColLighten, col_lighten);
    h.insert(Builtin::ColSetAlpha, col_set_alpha);
    h.insert(Builtin::ColGetAlpha, col_get_alpha);
    h.insert(Builtin::ColSetR, col_set_r);
    h.insert(Builtin::ColGetR, col_get_r);
    h.insert(Builtin::ColSetG, col_set_g);
    h.insert(Builtin::ColGetG, col_get_g);
    h.insert(Builtin::ColSetB, col_set_b);
    h.insert(Builtin::ColGetB, col_get_b);
    h.insert(Builtin::ColSetH, col_set_h);
    h.insert(Builtin::ColGetH, col_get_h);
    h.insert(Builtin::ColSetS, col_set_s);
    h.insert(Builtin::ColGetS, col_get_s);
    h.insert(Builtin::ColSetL, col_set_l);
    h.insert(Builtin::ColGetL, col_get_l);
    h.insert(Builtin::ColSetA, col_set_a);
    h.insert(Builtin::ColGetA, col_get_a);
    h.insert(Builtin::ColSetV, col_set_v);
    h.insert(Builtin::ColGetV, col_get_v);
    h.insert(Builtin::ColBuildProcedural, col_build_procedural);
    // BIND("col/build-bezier", col_build_bezier);
    h.insert(Builtin::ColValue, col_value);

    // --------------------------------------------------
    // math
    // --------------------------------------------------
    h.insert(Builtin::MathDistance, math_distance);
    h.insert(Builtin::MathNormal, math_normal);
    h.insert(Builtin::MathClamp, math_clamp);
    h.insert(Builtin::MathRadiansDegrees, math_radians_to_degrees);
    h.insert(Builtin::MathCos, math_cos);
    h.insert(Builtin::MathSin, math_sin);

    // --------------------------------------------------
    // prng
    // --------------------------------------------------
    h.insert(Builtin::PrngBuild, prng_build);
    h.insert(Builtin::PrngValues, prng_values);
    h.insert(Builtin::PrngValue, prng_value);
    h.insert(Builtin::PrngPerlin, prng_perlin);

    // --------------------------------------------------
    // interp
    // --------------------------------------------------
    h.insert(Builtin::InterpBuild, interp_build);
    h.insert(Builtin::InterpValue, interp_value);
    h.insert(Builtin::InterpCos, interp_cos);
    h.insert(Builtin::InterpSin, interp_sin);
    h.insert(Builtin::InterpBezier, interp_bezier);
    h.insert(Builtin::InterpBezierTangent, interp_bezier_tangent);
    h.insert(Builtin::InterpRay, interp_ray);
    h.insert(Builtin::InterpLine, interp_line);
    h.insert(Builtin::InterpCircle, interp_circle);

    // --------------------------------------------------
    // path
    // --------------------------------------------------
    h.insert(Builtin::PathLinear, path_linear);
    h.insert(Builtin::PathCircle, path_circle);
    h.insert(Builtin::PathSpline, path_spline);
    h.insert(Builtin::PathBezier, path_bezier);

    // --------------------------------------------------
    // repeat
    // --------------------------------------------------
    h.insert(Builtin::RepeatSymmetryVertical, repeat_symmetry_vertical);
    h.insert(
        Builtin::RepeatSymmetryHorizontal,
        repeat_symmetry_horizontal,
    );
    h.insert(Builtin::RepeatSymmetry4, repeat_symmetry_4);
    h.insert(Builtin::RepeatSymmetry8, repeat_symmetry_8);
    h.insert(Builtin::RepeatRotate, repeat_rotate);
    h.insert(Builtin::RepeatRotateMirrored, repeat_mirrored);

    // --------------------------------------------------
    // focal
    // --------------------------------------------------
    h.insert(Builtin::FocalBuildPoint, focal_build_point);
    h.insert(Builtin::FocalBuildHLine, focal_build_hline);
    h.insert(Builtin::FocalBuildVLine, focal_build_vline);
    h.insert(Builtin::FocalValue, focal_value);

    // --------------------------------------------------
    // gen
    // --------------------------------------------------
    h.insert(Builtin::GenStrayInt, gen_stray_int);
    h.insert(Builtin::GenStray, gen_stray);
    h.insert(Builtin::GenStray2D, gen_stray_2d);
    h.insert(Builtin::GenStray3D, gen_stray_3d);
    h.insert(Builtin::GenStray4D, gen_stray_4d);
    h.insert(Builtin::GenInt, gen_int);
    h.insert(Builtin::GenScalar, gen_scalar);
    h.insert(Builtin::Gen2D, gen_2d);
    h.insert(Builtin::GenSelect, gen_select);
    h.insert(Builtin::GenCol, gen_col);

    h
}

struct ArgBindings<'a> {
    i32kw_to_var: HashMap<i32, Option<&'a Var>>,
}

impl<'a> ArgBindings<'a> {
    fn create(
        vm: &'a Vm,
        num_args: usize,
        binding_decls: Vec<(Keyword, Option<&'a Var>)>,
    ) -> Result<Self> {
        let mut i32kw_to_var: HashMap<i32, Option<&Var>> = HashMap::new();

        for bd in binding_decls {
            i32kw_to_var.insert(bd.0 as i32, bd.1);
        }

        let mut args_pointer = vm.sp - (num_args * 2);
        for _ in 0..num_args {
            let label = &vm.stack[args_pointer];
            let value = &vm.stack[args_pointer + 1];
            args_pointer += 2;

            if let Var::Int(iname) = label {
                i32kw_to_var.insert(*iname, Some(value));
            }
        }

        Ok(ArgBindings { i32kw_to_var })
    }

    fn get_kw(&self, kw: Keyword) -> Result<Keyword> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Keyword(kw) = var {
                    return Ok(*kw);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_kw".to_string()))
    }

    fn get_option_kw(&self, kw: Keyword) -> Option<Keyword> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Keyword(kw) = var {
                    return Some(*kw);
                }
            }
        }
        None
    }

    fn get_i32(&self, kw: Keyword) -> Result<i32> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Int(i) = var {
                    return Ok(*i);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_i32".to_string()))
    }

    fn get_option_i32(&self, kw: Keyword) -> Option<i32> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Int(i) = var {
                    return Some(*i);
                }
            }
        }
        None
    }

    fn get_f32(&self, kw: Keyword) -> Result<f32> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Float(f) = var {
                    return Ok(*f);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_f32".to_string()))
    }

    fn get_option_f32(&self, kw: Keyword) -> Option<f32> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Float(f) = var {
                    return Some(*f);
                }
            }
        }
        None
    }

    fn get_v2d(&self, kw: Keyword) -> Result<(f32, f32)> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::V2D(x, y) = var {
                    return Ok((*x, *y));
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_v2d".to_string()))
    }

    fn get_option_v2d(&self, kw: Keyword) -> Option<(f32, f32)> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::V2D(x, y) = var {
                    return Some((*x, *y));
                }
            }
        }
        None
    }

    fn get_usize(&self, kw: Keyword) -> Result<usize> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Float(f) = var {
                    return Ok(*f as usize);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_usize".to_string()))
    }

    fn get_col(&self, kw: Keyword) -> Result<Colour> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Colour(col) = var {
                    return Ok(*col);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_col".to_string()))
    }

    fn get_option_col(&self, kw: Keyword) -> Option<Colour> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Colour(col) = var {
                    return Some(*col);
                }
            }
        }
        None
    }

    // get kw_preferred, if it doesn't exist fallback to kw_fallback
    fn get_preferred_col(&self, kw_preferred: Keyword, kw_fallback: Keyword) -> Result<Colour> {
        if let Some(var) = self.i32kw_to_var.get(&(kw_preferred as i32)) {
            if let Some(var) = var {
                if let Var::Colour(col) = var {
                    return Ok(*col);
                }
            } else if let Some(var) = self.i32kw_to_var.get(&(kw_fallback as i32)) {
                if let Some(var) = var {
                    if let Var::Colour(col) = var {
                        return Ok(*col);
                    }
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_preferred_col".to_string()))
    }

    // assume that the value in the hashmap isn't None
    fn get_var(&self, kw: Keyword) -> Result<&Var> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                return Ok(var);
            }
        }
        Err(Error::Bind("ArgBindings::get_var".to_string()))
    }

    // returns the entire Option<&Var> value from the hashmap
    fn get_option_var(&self, kw: Keyword) -> Option<&Var> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            return *var;
        }
        None
    }

    fn get_brush(&self, kw: Keyword) -> Result<BrushType> {
        if let Some(var) = self.i32kw_to_var.get(&(kw as i32)) {
            if let Some(var) = var {
                if let Var::Keyword(n) = var {
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
                    return Ok(brush);
                }
            }
        }
        Err(Error::Bind("ArgBindings::get_brush".to_string()))
    }
}

macro_rules! arg_usize {
    ($kw:expr, $val:expr) => {
        ($kw, Some(&Var::Float($val as f32)))
    };
}

macro_rules! arg_f32 {
    ($kw:expr, $val:expr) => {
        ($kw, Some(&Var::Float($val)))
    };
}

macro_rules! arg_v2d {
    ($kw:expr, $val:expr) => {
        ($kw, Some(&Var::V2D($val.0, $val.1)))
    };
}

macro_rules! arg_kw {
    ($kw:expr, $val:expr) => {
        ($kw, Some(&Var::Keyword($val)))
    };
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

// pub fn dummy_fn(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
//     let _bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Vector, None)])?;
//     Ok(Var::Bool(false))
// }

pub fn vector_length(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Vector, None)])?;

    if let Some(var) = bindings.get_option_var(Keyword::Vector) {
        if let Var::Vector(vs) = var {
            let len = vs.len();
            return Ok(Var::Int(len as i32));
        }
    }

    Err(Error::Bind(
        "vector_length requires vector argument".to_string(),
    ))
}

pub fn probe(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    // slightly different way of dealing with arguments.
    // This is because debug_str_append requires a mutable reference
    // and the bindings use an immutable reference to vm.

    let (scalar, v, ws) = {
        let bindings = ArgBindings::create(
            vm,
            num_args,
            vec![
                (Keyword::Scalar, None),
                (Keyword::Vector, None),
                (Keyword::WorldSpace, None),
            ],
        )?;

        (
            bindings.get_option_f32(Keyword::Scalar),
            bindings.get_option_v2d(Keyword::Vector),
            bindings.get_option_v2d(Keyword::WorldSpace),
        )
    };

    if let Some(f) = scalar {
        vm.debug_str_append(&format!("{}", f));
    }

    if let Some((x, y)) = v {
        vm.debug_str_append(&format!("({},{})", x, y));
    }

    if let Some((x, y)) = ws {
        if let Some(matrix) = vm.matrix_stack.peek() {
            let (nx, ny) = matrix.transform_vec2(x, y);
            vm.debug_str_append(&format!("({},{})", nx, ny));
        }
    }

    Ok(Var::Bool(true))
}

pub fn line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Width, 4.0),
            arg_v2d!(Keyword::From, (10.0, 10.0)),
            arg_v2d!(Keyword::To, (900.0, 900.0)),
            (Keyword::FromColour, None),
            (Keyword::ToColour, None),
            (Keyword::Colour, Some(&default_colour)),
            (Keyword::Brush, Some(&Var::Keyword(Keyword::BrushFlat))),
            arg_usize!(Keyword::BrushSubtype, 0),
        ],
    )?;

    let from_col = bindings.get_preferred_col(Keyword::FromColour, Keyword::Colour)?;
    let to_col = bindings.get_preferred_col(Keyword::ToColour, Keyword::Colour)?;

    if let Ok(from_c) = from_col.convert(ColourFormat::Rgb) {
        if let Ok(to_c) = to_col.convert(ColourFormat::Rgb) {
            vm.render_line(
                bindings.get_v2d(Keyword::From)?,
                bindings.get_v2d(Keyword::To)?,
                bindings.get_f32(Keyword::Width)?,
                &from_c,
                &to_c,
                bindings.get_brush(Keyword::Brush)?,
                bindings.get_usize(Keyword::BrushSubtype)?,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn rect(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Width, 4.0),
            arg_f32!(Keyword::Height, 10.0),
            arg_v2d!(Keyword::Position, (10.0, 10.0)),
            (Keyword::Colour, Some(&default_colour)),
        ],
    )?;

    if let Ok(rgb) = bindings
        .get_col(Keyword::Colour)?
        .convert(ColourFormat::Rgb)
    {
        vm.render_rect(
            bindings.get_v2d(Keyword::Position)?,
            bindings.get_f32(Keyword::Width)?,
            bindings.get_f32(Keyword::Height)?,
            &rgb,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Width, 4.0),
            arg_f32!(Keyword::Height, 10.0),
            arg_v2d!(Keyword::Position, (10.0, 10.0)),
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::Tessellation, 10.0),
            (Keyword::Radius, None),
        ],
    )?;

    // if the radius has been defined then it overrides the width and height parameters
    let (width, height) = if let Some(r) = bindings.get_option_f32(Keyword::Radius) {
        (r, r)
    } else {
        (
            bindings.get_f32(Keyword::Width)?,
            bindings.get_f32(Keyword::Height)?,
        )
    };

    if let Ok(rgb) = bindings
        .get_col(Keyword::Colour)?
        .convert(ColourFormat::Rgb)
    {
        vm.render_circle(
            bindings.get_v2d(Keyword::Position)?,
            width,
            height,
            &rgb,
            bindings.get_usize(Keyword::Tessellation)?,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn circle_slice(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Width, 4.0),
            arg_f32!(Keyword::Height, 10.0),
            arg_v2d!(Keyword::Position, (10.0, 10.0)),
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::Tessellation, 10.0),
            (Keyword::Radius, None),
            arg_f32!(Keyword::AngleStart, 0.0),
            arg_f32!(Keyword::AngleEnd, 10.0),
            arg_f32!(Keyword::InnerWidth, 1.0),
            arg_f32!(Keyword::InnerHeight, 1.0),
        ],
    )?;

    // if the radius has been defined then it overrides the width and height parameters
    let (width, height) = if let Some(r) = bindings.get_option_f32(Keyword::Radius) {
        (r, r)
    } else {
        (
            bindings.get_f32(Keyword::Width)?,
            bindings.get_f32(Keyword::Height)?,
        )
    };

    if let Ok(rgb) = bindings
        .get_col(Keyword::Colour)?
        .convert(ColourFormat::Rgb)
    {
        vm.render_circle_slice(
            bindings.get_v2d(Keyword::Position)?,
            width,
            height,
            &rgb,
            bindings.get_usize(Keyword::Tessellation)?,
            bindings.get_f32(Keyword::AngleStart)?,
            bindings.get_f32(Keyword::AngleEnd)?,
            bindings.get_f32(Keyword::InnerWidth)?,
            bindings.get_f32(Keyword::InnerHeight)?,
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
            vm.render_quadratic(
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                &rgb_c,
                tessellation.unwrap() as usize,
                // TODO
                BrushType::Flat,
                0, // bindings.get_brush(Keyword::Brush)?,
                   // bindings.get_usize(Keyword::BrushSubtype)?
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
            vm.render_bezier(
                &co,
                width_start,
                width_end,
                mapping,
                t_start.unwrap(),
                t_end.unwrap(),
                &rgb_c,
                tessellation.unwrap() as usize,
                // TODO
                BrushType::Flat,
                0, //bindings.get_brush(Keyword::Brush)?,
                   //bindings.get_usize(Keyword::BrushSubtype)?
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

    let co = array_f32_8_from_vec(coords.unwrap());

    if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
        vm.render_bezier_bulging(
            &co,
            line_width.unwrap(),
            t_start.unwrap(),
            t_end.unwrap(),
            &rgb_c,
            tessellation.unwrap() as usize,
            // TODO
            BrushType::Flat,
            0, // bindings.get_brush(Keyword::Brush)?,
               // bindings.get_usize(Keyword::BrushSubtype)?
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

    let co = array_f32_8_from_vec(coords.unwrap());

    let maybe_mapping = easing_from_keyword(line_width_mapping.unwrap());
    if let Some(mapping) = maybe_mapping {
        if let Ok(rgb_c) = colour.unwrap().convert(ColourFormat::Rgb) {
            vm.render_stroked_bezier(
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
                // TODO
                BrushType::Flat,
                0, // bindings.get_brush(Keyword::Brush)?,
                   // bindings.get_usize(Keyword::BrushSubtype)?
            )?;

            return Ok(Var::Bool(true));
        }
    }

    Ok(Var::Bool(false))
}

pub fn stroked_bezier_rect(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Position, (100.0, 100.0)),
            arg_f32!(Keyword::Width, 80.0),
            arg_f32!(Keyword::Height, 600.0),
            arg_f32!(Keyword::Volatility, 30.0),
            arg_f32!(Keyword::Overlap, 0.0),
            arg_f32!(Keyword::Iterations, 10.0),
            arg_f32!(Keyword::Seed, 0.0),
            arg_usize!(Keyword::Tessellation, 15),
            arg_usize!(Keyword::StrokeTessellation, 10),
            arg_f32!(Keyword::StrokeNoise, 25.0),
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::ColourVolatility, 0.0),
            (Keyword::Brush, Some(&Var::Keyword(Keyword::BrushFlat))),
            arg_usize!(Keyword::BrushSubtype, 0),
        ],
    )?;

    if let Ok(rgb) = bindings
        .get_col(Keyword::Colour)?
        .convert(ColourFormat::Rgb)
    {
        vm.render_stroked_bezier_rect(
            bindings.get_v2d(Keyword::Position)?,
            bindings.get_f32(Keyword::Width)?,
            bindings.get_f32(Keyword::Height)?,
            bindings.get_f32(Keyword::Volatility)?,
            bindings.get_f32(Keyword::Overlap)?,
            bindings.get_f32(Keyword::Iterations)?,
            bindings.get_f32(Keyword::Seed)? as i32,
            bindings.get_usize(Keyword::Tessellation)?,
            bindings.get_usize(Keyword::StrokeTessellation)?,
            bindings.get_f32(Keyword::StrokeNoise)?,
            &rgb,
            bindings.get_f32(Keyword::ColourVolatility)?,
            bindings.get_brush(Keyword::Brush)?,
            bindings.get_usize(Keyword::BrushSubtype)?,
        )?;

        Ok(Var::Bool(true))
    } else {
        Ok(Var::Bool(false))
    }
}

pub fn translate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![arg_v2d!(Keyword::Vector, (0.0, 0.0))])?;

    let (x, y) = bindings.get_v2d(Keyword::Vector)?;
    vm.matrix_stack.translate(x, y);

    Ok(Var::Bool(true))
}

pub fn rotate(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![arg_f32!(Keyword::Angle, 0.0)])?;

    vm.matrix_stack
        .rotate(mathutil::deg_to_rad(bindings.get_f32(Keyword::Angle)?));

    Ok(Var::Bool(true))
}

pub fn scale(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Vector, (1.0, 1.0)),
            (Keyword::Scalar, None),
        ],
    )?;

    if let Some(s) = bindings.get_option_f32(Keyword::Scalar) {
        vm.matrix_stack.scale(s, s);
    } else {
        let (sx, sy) = bindings.get_v2d(Keyword::Vector)?;
        vm.matrix_stack.scale(sx, sy);
    }

    Ok(Var::Bool(true))
}

pub fn col_convert(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_kw!(Keyword::Format, Keyword::Rgb),
            (Keyword::Colour, Some(&default_colour)),
        ],
    )?;

    if let Some(fmt) = ColourFormat::from_keyword(bindings.get_kw(Keyword::Format)?) {
        let colour = bindings.get_col(Keyword::Colour)?;
        let col = colour.convert(fmt)?;

        Ok(Var::Colour(col))
    } else {
        Err(Error::Bind("col_convert".to_string()))
    }
}

pub fn col_rgb(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::R, 0.0),
            arg_f32!(Keyword::G, 0.0),
            arg_f32!(Keyword::B, 0.0),
            arg_f32!(Keyword::Alpha, 1.0),
        ],
    )?;

    Ok(Var::Colour(Colour::new(
        ColourFormat::Rgb,
        bindings.get_f32(Keyword::R)?,
        bindings.get_f32(Keyword::G)?,
        bindings.get_f32(Keyword::B)?,
        bindings.get_f32(Keyword::Alpha)?,
    )))
}

pub fn col_hsl(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::H, 0.0),
            arg_f32!(Keyword::S, 0.0),
            arg_f32!(Keyword::L, 0.0),
            arg_f32!(Keyword::Alpha, 1.0),
        ],
    )?;

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsl,
        bindings.get_f32(Keyword::H)?,
        bindings.get_f32(Keyword::S)?,
        bindings.get_f32(Keyword::L)?,
        bindings.get_f32(Keyword::Alpha)?,
    )))
}

pub fn col_hsluv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::H, 0.0),
            arg_f32!(Keyword::S, 0.0),
            arg_f32!(Keyword::L, 0.0),
            arg_f32!(Keyword::Alpha, 1.0),
        ],
    )?;

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsluv,
        bindings.get_f32(Keyword::H)?,
        bindings.get_f32(Keyword::S)?,
        bindings.get_f32(Keyword::L)?,
        bindings.get_f32(Keyword::Alpha)?,
    )))
}

pub fn col_hsv(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::H, 0.0),
            arg_f32!(Keyword::S, 0.0),
            arg_f32!(Keyword::V, 0.0),
            arg_f32!(Keyword::Alpha, 1.0),
        ],
    )?;

    Ok(Var::Colour(Colour::new(
        ColourFormat::Hsv,
        bindings.get_f32(Keyword::H)?,
        bindings.get_f32(Keyword::S)?,
        bindings.get_f32(Keyword::V)?,
        bindings.get_f32(Keyword::Alpha)?,
    )))
}

pub fn col_lab(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::L, 0.0),
            arg_f32!(Keyword::A, 0.0),
            arg_f32!(Keyword::B, 0.0),
            arg_f32!(Keyword::Alpha, 1.0),
        ],
    )?;

    Ok(Var::Colour(Colour::new(
        ColourFormat::Lab,
        bindings.get_f32(Keyword::L)?,
        bindings.get_f32(Keyword::A)?,
        bindings.get_f32(Keyword::B)?,
        bindings.get_f32(Keyword::Alpha)?,
    )))
}

pub fn col_complementary(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings =
        ArgBindings::create(vm, num_args, vec![(Keyword::Colour, Some(&default_colour))])?;

    let col = bindings.get_col(Keyword::Colour)?;
    let c1 = col.complementary()?;

    Ok(Var::Colour(c1))
}

pub fn col_split_complementary(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings =
        ArgBindings::create(vm, num_args, vec![(Keyword::Colour, Some(&default_colour))])?;

    let col = bindings.get_col(Keyword::Colour)?;
    let (col1, col2) = col.split_complementary()?;

    Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]))
}

pub fn col_analagous(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings =
        ArgBindings::create(vm, num_args, vec![(Keyword::Colour, Some(&default_colour))])?;

    let col = bindings.get_col(Keyword::Colour)?;
    let (col1, col2) = col.analagous()?;

    Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]))
}

pub fn col_triad(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings =
        ArgBindings::create(vm, num_args, vec![(Keyword::Colour, Some(&default_colour))])?;

    let col = bindings.get_col(Keyword::Colour)?;
    let (col1, col2) = col.triad()?;

    Ok(Var::Vector(vec![Var::Colour(col1), Var::Colour(col2)]))
}

pub fn col_darken(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::Value, 0.0),
        ],
    )?;

    let col = bindings.get_col(Keyword::Colour)?;
    let darkened = col.darken(bindings.get_f32(Keyword::Value)?)?;

    Ok(Var::Colour(darkened))
}

pub fn col_lighten(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::Value, 0.0),
        ],
    )?;

    let col = bindings.get_col(Keyword::Colour)?;
    let lightened = col.lighten(bindings.get_f32(Keyword::Value)?)?;

    Ok(Var::Colour(lightened))
}

pub fn col_set_elem(idx: usize, vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            (Keyword::Colour, Some(&default_colour)),
            arg_f32!(Keyword::Value, 0.0),
        ],
    )?;

    let col = bindings.get_col(Keyword::Colour)?;
    match idx {
        0 => Ok(Var::Colour(Colour::new(
            col.format,
            bindings.get_f32(Keyword::Value)?,
            col.e1,
            col.e2,
            col.e3,
        ))),
        1 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            bindings.get_f32(Keyword::Value)?,
            col.e2,
            col.e3,
        ))),
        2 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            col.e1,
            bindings.get_f32(Keyword::Value)?,
            col.e3,
        ))),
        3 => Ok(Var::Colour(Colour::new(
            col.format,
            col.e0,
            col.e1,
            col.e2,
            bindings.get_f32(Keyword::Value)?,
        ))),
        _ => Err(Error::Bind("col_set_elem::idx out of range".to_string())),
    }
}

pub fn col_get_elem(idx: usize, vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let default_colour = Var::Colour(Default::default());
    let bindings =
        ArgBindings::create(vm, num_args, vec![(Keyword::Colour, Some(&default_colour))])?;

    let col = bindings.get_col(Keyword::Colour)?;
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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Vec1, (0.0, 0.0)),
            arg_v2d!(Keyword::Vec2, (0.0, 0.0)),
        ],
    )?;

    let v1 = bindings.get_v2d(Keyword::Vec1)?;
    let v2 = bindings.get_v2d(Keyword::Vec2)?;
    let distance = mathutil::distance_v2(v1.0, v1.1, v2.0, v2.1);

    Ok(Var::Float(distance))
}

pub fn math_normal(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Vec1, (0.0, 0.0)),
            arg_v2d!(Keyword::Vec2, (0.0, 0.0)),
        ],
    )?;

    let v1 = bindings.get_v2d(Keyword::Vec1)?;
    let v2 = bindings.get_v2d(Keyword::Vec2)?;
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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Value, 0.0),
            arg_f32!(Keyword::Min, 0.0),
            arg_f32!(Keyword::Max, 0.0),
        ],
    )?;

    let res = mathutil::clamp(
        bindings.get_f32(Keyword::Value)?,
        bindings.get_f32(Keyword::Min)?,
        bindings.get_f32(Keyword::Max)?,
    );
    Ok(Var::Float(res))
}

pub fn math_radians_to_degrees(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![arg_f32!(Keyword::Angle, 0.0)])?;
    let res = mathutil::rad_to_deg(bindings.get_f32(Keyword::Angle)?);

    Ok(Var::Float(res))
}

pub fn math_cos(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![arg_f32!(Keyword::Angle, 0.0)])?;
    let res = bindings.get_f32(Keyword::Angle)?.cos();

    Ok(Var::Float(res))
}

pub fn math_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![arg_f32!(Keyword::Angle, 0.0)])?;
    let res = bindings.get_f32(Keyword::Angle)?.sin();

    Ok(Var::Float(res))
}

pub fn prng_build(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Seed, 1.0),
            arg_f32!(Keyword::Min, 0.0),
            arg_f32!(Keyword::Max, 1.0),
        ],
    )?;

    let prng_state_struct = prng::PrngStateStruct::new(
        bindings.get_f32(Keyword::Seed)? as i32,
        bindings.get_f32(Keyword::Min)?,
        bindings.get_f32(Keyword::Max)?,
    );

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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::X, 1.0),
            arg_f32!(Keyword::Y, 1.0),
            arg_f32!(Keyword::Z, 1.0),
        ],
    )?;

    let res = prng::perlin(
        bindings.get_f32(Keyword::X)?,
        bindings.get_f32(Keyword::Y)?,
        bindings.get_f32(Keyword::Z)?,
    );

    Ok(Var::Float(res))
}

pub fn interp_build(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::From, (0.0, 1.0)),
            arg_v2d!(Keyword::To, (0.0, 100.0)),
            arg_kw!(Keyword::Clamping, Keyword::False),
            arg_kw!(Keyword::Mapping, Keyword::Linear),
        ],
    )?;

    if let Some(mapping) = easing_from_keyword(bindings.get_kw(Keyword::Mapping)?) {
        let from = bindings.get_v2d(Keyword::From)?;
        let to = bindings.get_v2d(Keyword::To)?;
        let clamping = bindings.get_kw(Keyword::Clamping)? == Keyword::True;

        let from_m = mathutil::mc_m(from.0, 0.0, from.1, 1.0);
        let from_c = mathutil::mc_c(from.0, 0.0, from_m);
        let to_m = mathutil::mc_m(0.0, to.0, 1.0, to.1);
        let to_c = mathutil::mc_c(0.0, to.0, to_m);

        Ok(Var::InterpState(interp::InterpStateStruct {
            from_m,
            to_m,
            from_c,
            to_c,
            to,
            clamping,
            mapping,
        }))
    } else {
        Err(Error::Bind("interp_build".to_string()))
    }
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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Amplitude, 1.0),
            arg_f32!(Keyword::Frequency, 1.0),
            arg_f32!(Keyword::T, 1.0), // t goes from 0 to TAU
        ],
    )?;

    let res = interp::cos(
        bindings.get_f32(Keyword::Amplitude)?,
        bindings.get_f32(Keyword::Frequency)?,
        bindings.get_f32(Keyword::T)?,
    );

    Ok(Var::Float(res))
}

pub fn interp_sin(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_f32!(Keyword::Amplitude, 1.0),
            arg_f32!(Keyword::Frequency, 1.0),
            arg_f32!(Keyword::T, 1.0), // t goes from 0 to TAU
        ],
    )?;

    let res = interp::sin(
        bindings.get_f32(Keyword::Amplitude)?,
        bindings.get_f32(Keyword::Frequency)?,
        bindings.get_f32(Keyword::T)?,
    );

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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Point, (0.0, 0.0)),
            arg_v2d!(Keyword::Direction, (1000.0, 1000.0)),
            arg_f32!(Keyword::T, 1.0),
        ],
    )?;

    let (x, y) = interp::ray(
        bindings.get_v2d(Keyword::Point)?,
        bindings.get_v2d(Keyword::Direction)?,
        bindings.get_f32(Keyword::T)?,
    );

    Ok(Var::V2D(x, y))
}

pub fn interp_line(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::From, (0.0, 0.0)),
            arg_v2d!(Keyword::To, (0.0, 0.0)),
            arg_kw!(Keyword::Clamping, Keyword::False),
            arg_kw!(Keyword::Mapping, Keyword::Linear),
            arg_f32!(Keyword::T, 1.0),
        ],
    )?;

    if let Some(mapping) = easing_from_keyword(bindings.get_kw(Keyword::Mapping)?) {
        let from = bindings.get_v2d(Keyword::From)?;
        let to = bindings.get_v2d(Keyword::To)?;
        let clamping = bindings.get_kw(Keyword::Clamping)? == Keyword::True;
        let t = bindings.get_f32(Keyword::T)?;

        let x = interp::scalar(from.0, to.0, mapping, clamping, t);
        let y = interp::scalar(from.1, to.1, mapping, clamping, t);

        Ok(Var::V2D(x, y))
    } else {
        Err(Error::Bind("interp_line".to_string()))
    }
}

pub fn interp_circle(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Position, (0.0, 0.0)),
            arg_f32!(Keyword::Radius, 1.0),
            arg_f32!(Keyword::T, 0.0),
        ],
    )?;

    let (x, y) = interp::circle(
        bindings.get_v2d(Keyword::Position)?,
        bindings.get_f32(Keyword::Radius)?,
        bindings.get_f32(Keyword::T)?,
    );

    Ok(Var::V2D(x, y))
}

pub fn path_linear(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    // (path/linear fn: foo steps: 10 from: [0 0] to: [100 100])
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::From, (0.0, 0.0)),
            arg_v2d!(Keyword::To, (10.0, 10.0)),
            arg_f32!(Keyword::Steps, 10.0),
            arg_f32!(Keyword::TStart, 0.0),
            arg_f32!(Keyword::TEnd, 1.0),
            (Keyword::Fn, None),
            arg_kw!(Keyword::Mapping, Keyword::Linear),
        ],
    )?;

    if let Some(mapping) = easing_from_keyword(bindings.get_kw(Keyword::Mapping)?) {
        if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
            let fr = bindings.get_v2d(Keyword::From)?;
            let to = bindings.get_v2d(Keyword::To)?;

            path::linear(
                vm,
                program,
                fun as usize,
                bindings.get_f32(Keyword::Steps)? as i32,
                bindings.get_f32(Keyword::TStart)?,
                bindings.get_f32(Keyword::TEnd)?,
                fr.0,
                fr.1,
                to.0,
                to.1,
                mapping,
            )?;
        }
    }
    Ok(Var::Bool(true))
}

pub fn path_circle(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::Position, (0.0, 0.0)),
            arg_f32!(Keyword::Radius, 100.0),
            arg_f32!(Keyword::Steps, 10.0),
            arg_f32!(Keyword::TStart, 0.0),
            arg_f32!(Keyword::TEnd, 1.0),
            (Keyword::Fn, None),
            arg_kw!(Keyword::Mapping, Keyword::Linear),
        ],
    )?;

    if let Some(mapping) = easing_from_keyword(bindings.get_kw(Keyword::Mapping)?) {
        if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
            let pos = bindings.get_v2d(Keyword::Position)?;

            path::circular(
                vm,
                program,
                fun as usize,
                bindings.get_f32(Keyword::Steps)? as i32,
                bindings.get_f32(Keyword::TStart)?,
                bindings.get_f32(Keyword::TEnd)?,
                pos.0,
                pos.1,
                bindings.get_f32(Keyword::Radius)?,
                mapping,
            )?;
        }
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
                co,
                mapping,
            )?;
        }
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_vertical(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Fn, None)])?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::symmetry_vertical(vm, program, fun as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_horizontal(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Fn, None)])?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::symmetry_horizontal(vm, program, fun as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_4(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Fn, None)])?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::symmetry_4(vm, program, fun as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_symmetry_8(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Fn, None)])?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::symmetry_8(vm, program, fun as usize)?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_rotate(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![(Keyword::Fn, None), arg_usize!(Keyword::Copies, 3)],
    )?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::rotate(
            vm,
            program,
            fun as usize,
            bindings.get_usize(Keyword::Copies)?,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn repeat_mirrored(vm: &mut Vm, program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![(Keyword::Fn, None), arg_usize!(Keyword::Copies, 3)],
    )?;

    if let Some(fun) = bindings.get_option_i32(Keyword::Fn) {
        repeat::rotate_mirrored(
            vm,
            program,
            fun as usize,
            bindings.get_usize(Keyword::Copies)?,
        )?;
    }

    Ok(Var::Bool(true))
}

pub fn focal_build_generic(
    vm: &mut Vm,
    num_args: usize,
    focal_type: focal::FocalType,
) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_kw!(Keyword::Mapping, Keyword::Linear),
            arg_v2d!(Keyword::Position, (0.0, 0.0)),
            arg_f32!(Keyword::Distance, 1.0),
            arg_kw!(Keyword::TransformPosition, Keyword::False),
        ],
    )?;

    if let Some(mapping) = easing_from_keyword(bindings.get_kw(Keyword::Mapping)?) {
        Ok(Var::FocalState(focal::FocalStateStruct {
            focal_type,
            mapping,
            position: bindings.get_v2d(Keyword::Position)?,
            distance: bindings.get_f32(Keyword::Distance)?,
            transform_pos: bindings.get_kw(Keyword::TransformPosition)? == Keyword::True,
        }))
    } else {
        Err(Error::Bind("focal_build_generic".to_string()))
    }
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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![arg_f32!(Keyword::From, 1.0), arg_f32!(Keyword::By, 0.2)],
    )?;

    let from = bindings.get_f32(Keyword::From)?;
    let by = mathutil::absf(bindings.get_f32(Keyword::By)?);

    let value = vm.prng_state.prng_f32_range(from - by, from + by);

    Ok(Var::Float(value.floor()))
}

pub fn gen_stray(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![arg_f32!(Keyword::From, 1.0), arg_f32!(Keyword::By, 0.2)],
    )?;

    let from = bindings.get_f32(Keyword::From)?;
    let by = mathutil::absf(bindings.get_f32(Keyword::By)?);

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

    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![
            arg_v2d!(Keyword::From, (10.0, 10.0)),
            arg_v2d!(Keyword::By, (1.0, 1.0)),
        ],
    )?;

    let from = bindings.get_v2d(Keyword::From)?;
    let by = bindings.get_v2d(Keyword::By)?;

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
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![arg_f32!(Keyword::Min, 0.0), arg_f32!(Keyword::Max, 1000.0)],
    )?;

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(
        bindings.get_f32(Keyword::Min)?,
        bindings.get_f32(Keyword::Max)? + 1.0,
    );

    Ok(Var::Float(value.floor()))
}

pub fn gen_scalar(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![arg_f32!(Keyword::Min, 0.0), arg_f32!(Keyword::Max, 1.0)],
    )?;

    // pick a scalar between min and max
    let value = vm.prng_state.prng_f32_range(
        bindings.get_f32(Keyword::Min)?,
        bindings.get_f32(Keyword::Max)?,
    );

    Ok(Var::Float(value))
}

pub fn gen_2d(vm: &mut Vm, _program: &Program, num_args: usize) -> Result<Var> {
    let bindings = ArgBindings::create(
        vm,
        num_args,
        vec![arg_f32!(Keyword::Min, 0.0), arg_f32!(Keyword::Max, 1.0)],
    )?;

    let min = bindings.get_f32(Keyword::Min)?;
    let max = bindings.get_f32(Keyword::Max)?;
    let x = vm.prng_state.prng_f32_range(min, max);
    let y = vm.prng_state.prng_f32_range(min, max);

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
    let bindings = ArgBindings::create(vm, num_args, vec![(Keyword::Alpha, None)])?;

    let alpha = if let Some(alpha) = bindings.get_option_f32(Keyword::Alpha) {
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
