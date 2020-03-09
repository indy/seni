// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::context::Context;
use crate::ease::{easing, Easing};
use crate::error::Result;
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::mathutil::{bezier_point, quadratic_point, TAU};
use crate::program::Program;
use crate::vm::*;

// invoke a function with 3 args: step, position and t
fn invoke_function(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    step: f32,
    t: f32,
    x: f32,
    y: f32,
) -> Result<()> {
    let ip = vm.ip;

    let fn_info = &program.fn_info[fun];

    vm.function_call_default_arguments(context, program, fn_info)?;
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::N), step);
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::T), t);
    vm.function_set_argument_to_2d(fn_info, Iname::from(Keyword::Position), x, y);
    vm.function_call_body(context, program, fn_info)?;

    vm.ip = ip;

    Ok(())
}

pub fn linear(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    steps: i32,
    t_start: f32,
    t_end: f32,
    a_x: f32,
    a_y: f32,
    b_x: f32,
    b_y: f32,
    mapping: Easing,
) -> Result<()> {
    let unit: f32 = (t_end - t_start) / (steps as f32 - 1.0);

    for i in 0..steps {
        let step = i as f32;
        let t = easing(t_start + (i as f32 * unit), mapping);

        let x = a_x + (t * (b_x - a_x));
        let y = a_y + (t * (b_y - a_y));

        invoke_function(vm, context, program, fun, step, t, x, y)?;
    }

    Ok(())
}

pub fn circular(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    steps: i32,
    t_start: f32,
    t_end: f32,
    pos_x: f32,
    pos_y: f32,
    radius: f32,
    mapping: Easing,
) -> Result<()> {
    let unit = (t_end - t_start) / steps as f32;
    let unit_angle = unit * TAU;

    for i in 0..steps {
        let step = i as f32;
        let angle = (unit_angle * step) + (t_start * TAU);
        let vx = (angle.sin() * radius) + pos_x;
        let vy = (angle.cos() * radius) + pos_y;
        let t = easing(t_start + (unit * step), mapping);

        invoke_function(vm, context, program, fun, step, t, vx, vy)?;
    }

    Ok(())
}

pub fn spline(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    steps: i32,
    t_start: f32,
    t_end: f32,
    coords: [f32; 6],
    mapping: Easing,
) -> Result<()> {
    let unit = (t_end - t_start) / (steps as f32 - 1.0);

    for i in 0..steps {
        let t = easing(t_start + (i as f32 * unit), mapping);

        let x = quadratic_point(coords[0], coords[2], coords[4], t);
        let y = quadratic_point(coords[1], coords[3], coords[5], t);

        invoke_function(vm, context, program, fun, i as f32, t, x, y)?;
    }

    Ok(())
}

pub fn bezier(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    steps: i32,
    t_start: f32,
    t_end: f32,
    coords: [f32; 8],
    mapping: Easing,
) -> Result<()> {
    let unit = (t_end - t_start) / (steps as f32 - 1.0);

    for i in 0..steps {
        let t = easing(t_start + (i as f32 * unit), mapping);

        let x = bezier_point(coords[0], coords[2], coords[4], coords[6], t);
        let y = bezier_point(coords[1], coords[3], coords[5], coords[7], t);

        invoke_function(vm, context, program, fun, i as f32, t, x, y)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_invocations() {
        probe_has_scalars(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe scalar: n))
                          (path/linear from: [10 10]
                                       to: [50 50]
                                       fn: (address-of point)
                                       steps: 5)",
            [0.0, 1.0, 2.0, 3.0, 4.0].to_vec(),
        );
    }

    #[test]
    fn test_linear() {
        probe_has_scalars(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe scalar: t))
                          (path/linear from: [10 10]
                                       to: [50 50]
                                       fn: (address-of point)
                                       steps: 5)",
            [0.0, 0.25, 0.5, 0.75, 1.0].to_vec(),
        );

        probe_has_scalars(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe scalar: t))
                          (path/linear from: [10 10]
                                       to: [50 50]
                                       fn: (address-of point)
                                       mapping: ease/quick
                                       steps: 5)",
            [0.0, 0.15625, 0.5, 0.84375, 1.0].to_vec(),
        );

        probe_has_scalars_v2(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe vector: position))
                          (path/linear from: [10 10]
                                       to: [50 50]
                                       fn: (address-of point)
                                       steps: 5)",
            [
                (10.0, 10.0),
                (20.0, 20.0),
                (30.0, 30.0),
                (40.0, 40.0),
                (50.0, 50.0),
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_circular() {
        probe_has_scalars_v2(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe vector: position))
                          (path/circle position: [0 0]
                                       radius: 100
                                       fn: (address-of point)
                                       steps: 8)",
            [
                (0.0, 100.0),
                (70.71068, 70.71068),
                (100.0, -0.000004371139),
                (70.71068, -70.71068),
                (-0.000008742278, -100.0),
                (-70.710686, -70.71066),
                (-100.0, 0.0000011924881),
                (-70.710655, 70.7107),
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_spline() {
        probe_has_scalars_v2(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe vector: position))
                          (path/spline coords: [[100 500] [300 500] [500 500]]
                                       fn: (address-of point)
                                       steps: 8)",
            [
                (100.0, 500.0),
                (157.14285, 500.0),
                (214.28572, 500.0),
                (271.4286, 500.0),
                (328.57144, 500.0),
                (385.7143, 500.0),
                (442.85718, 500.0),
                (500.0, 500.0),
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_bezier() {
        probe_has_scalars_v2(
            "(fn (point position: [500 500]
                                    n: 1
                                    t: 0.2)
                              (probe vector: position))
                          (path/bezier coords: [[100 500] [300 300] [500 800] [700 500]]
                                       fn: (address-of point)
                                       steps: 8)",
            [
                (100.0, 500.0),
                (185.71431, 452.76974),
                (271.42862, 465.01465),
                (357.14288, 510.49567),
                (442.85718, 562.97375),
                (528.5714, 596.2099),
                (614.28577, 583.965),
                (700.0, 500.0),
            ]
            .to_vec(),
        );
    }
}
