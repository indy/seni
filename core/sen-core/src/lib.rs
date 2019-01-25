#![cfg_attr(
    feature = "cargo-clippy",
    allow(
        clippy::many_single_char_names,
        clippy::excessive_precision,
        clippy::too_many_arguments
    )
)]
#![allow(dead_code)]
// todo: remove crate wide allowing of dead_code

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

mod colour;
mod ease;
mod focal;
mod interp;
mod keywords;
mod lexer;
mod mathutil;
mod matrix;
mod native;
mod opcodes;
mod path;
mod placeholder;
mod prng;
mod repeat;
mod uvmapper;

pub mod compiler;
pub mod error;
pub mod geometry;
pub mod parser;
pub mod vm;

use crate::error::*;
use crate::parser::*;
use crate::vm::*;

pub use crate::compiler::*;

pub fn compile_to_render_packets(vm: &mut Vm, s: &str) -> Result<i32> {
    let env = Env::new();

    let (ast, _word_lut) = parse(s)?;
    let program = compile_program(&ast)?;

    vm.reset();

    // todo: cache the preamble program

    // setup the env with the global variables in preamble
    let preamble = compile_preamble()?;
    vm.interpret(&env, &preamble)?;

    vm.ip = 0;

    vm.interpret(&env, &program)?;

    Ok(vm.geometry.get_num_render_packets() as i32)
}

pub fn compile_and_execute(s: &str) -> Result<Var> {
    let mut vm = Vm::new();
    let env = Env::new();

    // todo: cache the preamble program

    let (ast, _word_lut) = parse(s)?;
    let program = compile_program(&ast)?;

    // setup the env with the global variables in preamble
    let preamble = compile_preamble()?;
    vm.interpret(&env, &preamble)?;

    // can now run the user program
    vm.interpret(&env, &program)?;
    vm.top_stack_value()
}

pub fn compile_str(s: &str) -> Result<Program> {
    let (ast, _word_lut) = parse(s)?;
    compile_program(&ast)
}

pub fn sen_parse(s: &str) -> i32 {
    s.len() as i32
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn is_rendering_num_verts(vm: &mut Vm, s: &str, expected_num_verts: usize) {
        if let Ok(res) = compile_to_render_packets(vm, s) {
            assert_eq!(1, res);

            let num_verts = vm.get_render_packet_geo_len(0);
            assert_eq!(expected_num_verts * 8, num_verts);
        } else {
            assert_eq!(true, false);
        }
    }

    #[test]
    fn bug_running_preamble_crashed_vm() {
        // vm.ip wasn't being set to 0 in-between running the preamble and running the user's program
        let mut vm = Vm::new();
        let s = "(rect)";
        is_rendering_num_verts(&mut vm, &s, 4);
    }
}
