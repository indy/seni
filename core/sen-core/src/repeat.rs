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
use crate::error::*;
use crate::keywords::Keyword;
use crate::mathutil::*;
use crate::vm::*;

fn flip(vm: &mut Vm, program: &Program, fun: usize, sx: f32, sy: f32, copy_val: i32) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let copy = copy_val;
    vm.matrix_stack.push();
    {
        vm.function_call_default_arguments(program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Keyword::Copy as usize, copy as f32);
        vm.function_call_body(program, fn_info)?;
    }
    vm.matrix_stack.pop();
    vm.ip = ip;

    let copy = copy_val + 1;
    vm.matrix_stack.push();
    {
        vm.matrix_stack.scale(sx, sy);
        vm.function_call_default_arguments(program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Keyword::Copy as usize, copy as f32);
        vm.function_call_body(program, fn_info)?;
    }
    vm.matrix_stack.pop();
    vm.ip = ip;

    Ok(())
}

pub fn repeat_symmetry_vertical(vm: &mut Vm, program: &Program, fun: usize) -> Result<()> {
    flip(vm, program, fun, -1.0, 1.0, 0)
}

pub fn repeat_symmetry_horizontal(vm: &mut Vm, program: &Program, fun: usize) -> Result<()> {
    flip(vm, program, fun, 1.0, -1.0, 0)
}

pub fn repeat_symmetry_4_copy_offset(
    vm: &mut Vm,
    program: &Program,
    fun: usize,
    copy_offset: i32,
) -> Result<()> {
    vm.matrix_stack.push();
    flip(vm, program, fun, -1.0, 1.0, copy_offset)?;
    vm.matrix_stack.pop();

    vm.matrix_stack.push();
    vm.matrix_stack.scale(1.0, -1.0);
    flip(vm, program, fun, -1.0, 1.0, copy_offset + 2)?;
    vm.matrix_stack.pop();

    Ok(())
}

pub fn repeat_symmetry_4(vm: &mut Vm, program: &Program, fun: usize) -> Result<()> {
    repeat_symmetry_4_copy_offset(vm, program, fun, 0)
}

pub fn repeat_symmetry_8(vm: &mut Vm, program: &Program, fun: usize) -> Result<()> {
    vm.matrix_stack.push();
    repeat_symmetry_4_copy_offset(vm, program, fun, 0)?;
    vm.matrix_stack.pop();

    vm.matrix_stack.push();
    vm.matrix_stack.rotate(PI_BY_2);
    repeat_symmetry_4_copy_offset(vm, program, fun, 4)?;
    vm.matrix_stack.pop();

    Ok(())
}

pub fn repeat_rotate(vm: &mut Vm, program: &Program, fun: usize, copies: usize) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let delta = TAU / copies as f32;

    for i in 0..copies {
        let angle = delta * i as f32;

        vm.matrix_stack.push();
        vm.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Keyword::Angle as usize, rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Keyword::Copy as usize, i as f32);
        vm.function_call_body(program, fn_info)?;

        vm.matrix_stack.pop();
        vm.ip = ip;
    }

    Ok(())
}

pub fn repeat_rotate_mirrored(
    vm: &mut Vm,
    program: &Program,
    fun: usize,
    copies: usize,
) -> Result<()> {
    let fn_info = &program.fn_info[fun];
    let ip = vm.ip;

    let delta = TAU / copies as f32;

    for i in 0..copies {
        let angle = delta * i as f32;

        vm.matrix_stack.push();
        vm.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Keyword::Angle as usize, rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Keyword::Copy as usize, i as f32);
        vm.function_call_body(program, fn_info)?;

        vm.matrix_stack.pop();
        vm.ip = ip;
    }

    vm.matrix_stack.push();
    vm.matrix_stack.scale(-1.0, 1.0);

    for i in 0..copies {
        let angle = delta * i as f32;

        vm.matrix_stack.push();
        vm.matrix_stack.rotate(angle);

        vm.function_call_default_arguments(program, fn_info)?;
        vm.function_set_argument_to_f32(fn_info, Keyword::Angle as usize, -rad_to_deg(angle));
        vm.function_set_argument_to_f32(fn_info, Keyword::Copy as usize, (copies + i) as f32);
        vm.function_call_body(program, fn_info)?;

        vm.matrix_stack.pop();
        vm.ip = ip;
    }

    vm.matrix_stack.pop();

    Ok(())
}
