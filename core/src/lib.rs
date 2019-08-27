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

/*!
The core crate provides the basic functionality of the Seni system
*/

// this is just for documentation
pub mod seni_language;

// mod ast_checker;
mod bitmap;
mod bitmap_cache;
pub mod colour;
mod colour_palettes;
mod compiler;
mod context;
mod ease;
pub mod error;
mod focal;
mod gene;
mod geometry;
mod iname;
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
mod program;
mod repeat;
mod result;
mod rgb;
mod trait_list;
mod unparser;
mod uvmapper;
mod vm;

pub use crate::bitmap_cache::{BitmapCache, BitmapInfo};
pub use crate::compiler::{compile_preamble, compile_program, compile_program_with_genotype};
pub use crate::context::Context;
pub use crate::error::Error;
pub use crate::gene::{next_generation, Genotype};
pub use crate::geometry::Geometry;
pub use crate::packable::Packable;
pub use crate::parser::{parse, WordLut};
pub use crate::program::Program;
pub use crate::result::Result;
pub use crate::trait_list::TraitList;
pub use crate::unparser::{simplified_unparse, unparse};
pub use crate::vm::{VMProfiling, Var, Vm};

pub fn run_program_with_preamble(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
) -> Result<Var> {
    context.reset();
    vm.reset();

    // setup the env with the global variables in preamble
    let preamble = compile_preamble()?;
    vm.interpret(context, &preamble)?;

    // reset the ip and setup any profiling of the main program
    vm.init_for_main_program(&program, VMProfiling::Off)?;

    vm.interpret(context, &program)?;
    vm.top_stack_value()
}

pub fn program_from_source(s: &str) -> Result<Program> {
    let (ast, word_lut) = parse(s)?;
    let program = compile_program(&ast, &word_lut)?;

    Ok(program)
}

pub fn program_from_source_and_genotype(s: &str, genotype: &mut Genotype) -> Result<Program> {
    let (mut ast, word_lut) = parse(s)?;
    let program = compile_program_with_genotype(&mut ast, &word_lut, genotype)?;

    Ok(program)
}

pub fn bitmaps_to_transfer(program: &Program, context: &Context) -> Vec<String> {
    // the bitmaps used by the current program
    let bitmap_strings = program.data.bitmap_strings();

    // keep the names that aren't already in the bitmap_cache
    let bitmaps_to_transfer = context.bitmap_cache.uncached(bitmap_strings);

    bitmaps_to_transfer
}

pub fn build_traits(s: &str) -> Result<TraitList> {
    let (ast, word_lut) = parse(s)?;
    let trait_list = TraitList::compile(&ast, &word_lut)?;

    Ok(trait_list)
}

pub fn compile_and_execute(s: &str) -> Result<Var> {
    let mut vm: Vm = Default::default();
    let mut context: Context = Default::default();

    let program = program_from_source(s)?;

    run_program_with_preamble(&mut vm, &mut context, &program)
}

pub fn compile_and_execute_with_seeded_genotype(s: &str, seed: i32) -> Result<Var> {
    let mut vm: Vm = Default::default();
    let mut context: Context = Default::default();

    let trait_list = build_traits(s)?;
    let mut genotype = Genotype::build_from_seed(&trait_list, seed)?;
    let program = program_from_source_and_genotype(s, &mut genotype)?;

    run_program_with_preamble(&mut vm, &mut context, &program)
}

pub fn compile_str(s: &str) -> Result<Program> {
    let (ast, word_lut) = parse(s)?;

    compile_program(&ast, &word_lut)
}

pub fn sen_parse(s: &str) -> i32 {
    s.len() as i32
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn is_rendering_num_verts(
        vm: &mut Vm,
        context: &mut Context,
        s: &str,
        expected_num_verts: usize,
    ) {
        let program = program_from_source(s).unwrap();
        let _ = run_program_with_preamble(vm, context, &program).unwrap();

        assert_eq!(1, context.geometry.get_num_render_packets() as i32);

        let num_floats = context.get_render_packet_geo_len(0);
        let floats_per_vert = 8;
        assert_eq!(expected_num_verts, num_floats / floats_per_vert);
    }

    #[test]
    fn bug_running_preamble_crashed_vm() {
        // vm.ip wasn't being set to 0 in-between running the preamble and running the user's program
        let mut vm: Vm = Default::default();
        let mut context: Context = Default::default();
        let s = "(rect)";

        is_rendering_num_verts(&mut vm, &mut context, &s, 4);
    }

    // #[test]
    //     fn explore_1() {
    //         // vm.ip wasn't being set to 0 in-between running the preamble and running the user's program
    //         let mut vm: Vm = Default::default();
    //         let mut context: Context = Default::default();
    //         let s = "

    // (define
    //   coords1 {[[-3.718 -69.162]
    //             [463.301 -57.804]
    //             [456.097 -315.570]
    //             [318.683 -384.297]]
    //            (gen/2d min: -500 max: 500)}
    //   coords2 {[[424.112 19.779]
    //             [2.641 246.678]
    //             [-449.001 -79.842]
    //             [37.301 206.818]]
    //            (gen/2d min: -500 max: 500)}
    //   coords3 {[[83.331 -282.954]
    //             [92.716 -393.120]
    //             [426.686 -420.284]
    //             [-29.734 335.671]]
    //            (gen/2d min: -500 max: 500)}

    //   col-fn-1 (col/build-procedural preset: {transformers (gen/select from: col/procedural-fn-presets)}
    //                                  alpha: 0.08)
    //   col-fn-2 (col/build-procedural preset: {mars (gen/select from: col/procedural-fn-presets)}
    //                                  alpha: 0.08)
    //   col-fn-3 (col/build-procedural preset: {knight-rider (gen/select from: col/procedural-fn-presets)}
    //                                  alpha: 0.08)

    //   num-copies {28 (gen/int min: 1 max: 28)}
    //   squish (interp/build from: [0 (- num-copies 1)] to: [{1.2 (gen/scalar max: 2)} {0.45 (gen/scalar max: 2)}]))

    // (fn (draw angle: 0 copy: 0)
    //   (scale vector: [(interp/value from: squish t: copy) (interp/value from: squish t: copy)])
    //   (fence (t num: 200)
    //     (poly coords: [(interp/bezier t: t coords: coords1)
    //                    (interp/bezier t: t coords: coords2)
    //                    (interp/bezier t: t coords: coords3)]
    //           colours: [(col/value from: col-fn-1 t: t)
    //                     (col/value from: col-fn-2 t: t)
    //                     (col/value from: col-fn-3 t: t)])))

    // (fn (render)
    //   (rect position: [500 500]
    //         width: 1000
    //         height: 1000
    //         colour: (col/value from: col-fn-1 t: 0.5))
    //   (on-matrix-stack
    //     (translate vector: [(/ canvas/width 2) (/ canvas/height 2)])
    //     (scale vector: [0.8 0.8])
    //     (repeat/rotate-mirrored fn: (address-of draw)
    //                             copies: num-copies)))

    // (render)
    // ";
    //         is_rendering_num_verts(&mut vm, &mut context, &s, 1246);
    //     }
}
