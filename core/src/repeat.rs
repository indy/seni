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
use crate::error::Result;
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::mathutil::*;
use crate::program::Program;
use crate::vm::*;

fn flip(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    sx: f32,
    sy: f32,
    copy_val: i32,
) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let copy = copy_val;
    context.matrix_stack.push();
    {
        vm.function_call_default_arguments(context, program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Copy), copy as f32);
        vm.function_call_body(context, program, fn_info)?;
    }
    context.matrix_stack.pop();
    vm.ip = ip;

    let copy = copy_val + 1;
    context.matrix_stack.push();
    {
        context.matrix_stack.scale(sx, sy);
        vm.function_call_default_arguments(context, program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Copy), copy as f32);
        vm.function_call_body(context, program, fn_info)?;
    }
    context.matrix_stack.pop();
    vm.ip = ip;

    Ok(())
}

pub fn symmetry_vertical(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
) -> Result<()> {
    flip(vm, context, program, fun, -1.0, 1.0, 0)
}

pub fn symmetry_horizontal(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
) -> Result<()> {
    flip(vm, context, program, fun, 1.0, -1.0, 0)
}

pub fn symmetry_4_copy_offset(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    copy_offset: i32,
) -> Result<()> {
    context.matrix_stack.push();
    flip(vm, context, program, fun, -1.0, 1.0, copy_offset)?;
    context.matrix_stack.pop();

    context.matrix_stack.push();
    context.matrix_stack.scale(1.0, -1.0);
    flip(vm, context, program, fun, -1.0, 1.0, copy_offset + 2)?;
    context.matrix_stack.pop();

    Ok(())
}

pub fn symmetry_4(vm: &mut Vm, context: &mut Context, program: &Program, fun: usize) -> Result<()> {
    symmetry_4_copy_offset(vm, context, program, fun, 0)
}

pub fn symmetry_8(vm: &mut Vm, context: &mut Context, program: &Program, fun: usize) -> Result<()> {
    context.matrix_stack.push();
    symmetry_4_copy_offset(vm, context, program, fun, 0)?;
    context.matrix_stack.pop();

    context.matrix_stack.push();
    context.matrix_stack.rotate(PI_BY_2);
    symmetry_4_copy_offset(vm, context, program, fun, 4)?;
    context.matrix_stack.pop();

    Ok(())
}

pub fn rotate(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    copies: usize,
) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let delta = TAU / copies as f32;

    for i in 0..copies {
        let angle = delta * i as f32;

        context.matrix_stack.push();
        context.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(context, program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Angle), rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Copy), i as f32);
        vm.function_call_body(context, program, fn_info)?;

        context.matrix_stack.pop();

        vm.ip = ip;
    }

    Ok(())
}

pub fn rotate_mirrored(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    copies: usize,
) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let delta = TAU / copies as f32;

    for i in 0..copies {
        let angle = delta * i as f32;

        context.matrix_stack.push();
        context.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(context, program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Angle), rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Copy), i as f32);
        vm.function_call_body(context, program, fn_info)?;

        context.matrix_stack.pop();
        vm.ip = ip;
    }

    context.matrix_stack.push();
    context.matrix_stack.scale(-1.0, 1.0);

    for i in 0..copies {
        let angle = delta * i as f32;

        context.matrix_stack.push();
        context.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(context, program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Angle), -rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::Copy), (copies + i) as f32);
        vm.function_call_body(context, program, fn_info)?;

        context.matrix_stack.pop();
        vm.ip = ip;
    }

    context.matrix_stack.pop();

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_symmetry_vertical() {
        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [20 20]))
             (repeat/symmetry-vertical fn: (address-of f))",
            [(20.0, 20.0), (-20.0, 20.0)].to_vec(),
        );
    }

    #[test]
    fn test_symmetry_horizontal() {
        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [20 20]))
             (repeat/symmetry-horizontal fn: (address-of f))",
            [(20.0, 20.0), (20.0, -20.0)].to_vec(),
        );
    }

    #[test]
    fn test_symmetry_4() {
        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [10 20]))
             (repeat/symmetry-4 fn: (address-of f))",
            [(10.0, 20.0), (-10.0, 20.0), (10.0, -20.0), (-10.0, -20.0)].to_vec(),
        );
    }

    #[test]
    fn test_symmetry_8() {
        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [10 20]))
             (repeat/symmetry-8 fn: (address-of f))",
            [
                (10.0, 20.0),
                (-10.0, 20.0),
                (10.0, -20.0),
                (-10.0, -20.0),
                (-20.0, 9.999999),
                (-20.0, -10.000001),
                (20.0, 10.000001),
                (20.0, -9.999999),
            ]
            .to_vec(),
        );
    }

    #[test]
    fn shabba_test_rotate() {
        probe_has_scalars(
            "(fn (f angle: 0 copy: 0) (probe scalar: angle))
             (repeat/rotate fn: (address-of f) copies: 3)",
            [0.0, 120.0, 240.0].to_vec(),
        );
    }

    #[test]
    fn test_rotate() {
        probe_has_scalars(
            "(fn (f angle: 0 copy: 0) (probe scalar: angle))
             (repeat/rotate fn: (address-of f) copies: 3)",
            [0.0, 120.0, 240.0].to_vec(),
        );

        probe_has_scalars(
            "(fn (f angle: 0 copy: 0) (probe scalar: copy))
             (repeat/rotate fn: (address-of f) copies: 3)",
            [0.0, 1.0, 2.0].to_vec(),
        );

        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [0 1]))
             (repeat/rotate fn: (address-of f) copies: 3)",
            [
                (0.0, 1.0),
                (-0.8660254, -0.50000006),
                (0.86602545, -0.4999999),
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_rotate_mirrored() {
        probe_has_scalars(
            "(fn (f angle: 0 copy: 0) (probe scalar: angle))
             (repeat/rotate-mirrored fn: (address-of f) copies: 3)",
            [0.0, 120.0, 240.0, 0.0, -120.0, -240.0].to_vec(),
        );

        probe_has_scalars(
            "(fn (f angle: 0 copy: 0) (probe scalar: copy))
             (repeat/rotate-mirrored fn: (address-of f) copies: 3)",
            [0.0, 1.0, 2.0, 3.0, 4.0, 5.0].to_vec(),
        );

        probe_has_scalars_v2(
            "(fn (f) (probe worldspace: [0 1]))
             (repeat/rotate-mirrored fn: (address-of f) copies: 3)",
            [
                (0.0, 1.0),
                (-0.8660254, -0.50000006),
                (0.86602545, -0.4999999),
                (0.0, 1.0),
                (0.8660254, -0.50000006),
                (-0.86602545, -0.4999999),
            ]
            .to_vec(),
        );
    }
}
