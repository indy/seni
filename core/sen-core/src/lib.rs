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

/*!
The sen-core crate provides the basic functionality of the Seni system
*/

mod colour;
mod compiler;
mod ease;
pub mod error; // for native
mod focal;
mod gene;
mod geometry;
mod interp;
mod keywords;
mod lexer;
mod mathutil;
mod matrix;
mod native;
mod opcodes;
mod packable;
mod parser;
mod path;
mod prng;
mod repeat;
pub mod seni_language;
mod trait_list;
mod uvmapper;
mod vm;

use crate::compiler::{compile_preamble, compile_program, compile_program_with_genotype, Program};
use crate::error::*;
use crate::parser::*;
use crate::vm::*;

pub use crate::gene::Genotype;
pub use crate::trait_list::TraitList;
pub use crate::vm::{Env, Vm};
pub use crate::packable::Packable;

pub fn run_program_with_preamble(vm: &mut Vm, program: &Program) -> Result<Var> {
    let env = Env::new();

    vm.reset();

    // setup the env with the global variables in preamble
    let preamble = compile_preamble()?;
    vm.interpret(&env, &preamble)?;

    vm.ip = 0;

    vm.interpret(&env, &program)?;
    vm.top_stack_value()
}

pub fn compile_to_render_packets(vm: &mut Vm, s: &str) -> Result<i32> {
    let (ast, _word_lut) = parse(s)?;
    let program = compile_program(&ast)?;
    let _ = run_program_with_preamble(vm, &program)?;

    // todo: cache the preamble program

    Ok(vm.geometry.get_num_render_packets() as i32)
}

pub fn build_traits(s: &str) -> Result<TraitList> {
    let (ast, _) = parse(s)?;

    let trait_list = TraitList::compile(&ast)?;

    Ok(trait_list)
}

pub fn compile_and_execute(s: &str) -> Result<Var> {
    let mut vm = Vm::new();
    // todo: cache the preamble program
    let (ast, _word_lut) = parse(s)?;
    let program = compile_program(&ast)?;

    run_program_with_preamble(&mut vm, &program)
}

pub fn compile_and_execute_with_seeded_genotype(s: &str, seed: i32) -> Result<Var> {
    let mut vm = Vm::new();
    // todo: cache the preamble program
    let (mut ast, _word_lut) = parse(s)?;

    let trait_list = TraitList::compile(&ast)?;
    let mut genotype = Genotype::build_from_seed(&trait_list, seed)?;
    let program = compile_program_with_genotype(&mut ast, &mut genotype)?;

    run_program_with_preamble(&mut vm, &program)
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

            let num_floats = vm.get_render_packet_geo_len(0);
            let floats_per_vert = 8;
            assert_eq!(expected_num_verts, num_floats / floats_per_vert);
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

    // #[test]
    fn explore_1() {
        // vm.ip wasn't being set to 0 in-between running the preamble and running the user's program
        let mut vm = Vm::new();
        let s = "

(define
  coords1 {[[-3.718 -69.162]
            [463.301 -57.804]
            [456.097 -315.570]
            [318.683 -384.297]]
           (gen/2d min: -500 max: 500)}
  coords2 {[[424.112 19.779]
            [2.641 246.678]
            [-449.001 -79.842]
            [37.301 206.818]]
           (gen/2d min: -500 max: 500)}
  coords3 {[[83.331 -282.954]
            [92.716 -393.120]
            [426.686 -420.284]
            [-29.734 335.671]]
           (gen/2d min: -500 max: 500)}

  col-fn-1 (col/build-procedural preset: {transformers (gen/select from: col/procedural-fn-presets)}
                                 alpha: 0.08)
  col-fn-2 (col/build-procedural preset: {mars (gen/select from: col/procedural-fn-presets)}
                                 alpha: 0.08)
  col-fn-3 (col/build-procedural preset: {knight-rider (gen/select from: col/procedural-fn-presets)}
                                 alpha: 0.08)

  num-copies {28 (gen/int min: 1 max: 28)}
  squish (interp/build from: [0 (- num-copies 1)] to: [{1.2 (gen/scalar max: 2)} {0.45 (gen/scalar max: 2)}]))

(fn (draw angle: 0 copy: 0)
  (scale vector: [(interp/value from: squish t: copy) (interp/value from: squish t: copy)])
  (fence (t num: 200)
    (poly coords: [(interp/bezier t: t coords: coords1)
                   (interp/bezier t: t coords: coords2)
                   (interp/bezier t: t coords: coords3)]
          colours: [(col/value from: col-fn-1 t: t)
                    (col/value from: col-fn-2 t: t)
                    (col/value from: col-fn-3 t: t)])))

(fn (render)
  (rect position: [500 500]
        width: 1000
        height: 1000
        colour: (col/value from: col-fn-1 t: 0.5))
  (on-matrix-stack
    (translate vector: [(/ canvas/width 2) (/ canvas/height 2)])
    (scale vector: [0.8 0.8])
    (repeat/rotate-mirrored fn: (address-of draw)
                            copies: num-copies)))

(render)
";
        is_rendering_num_verts(&mut vm, &s, 1246);
    }
}
