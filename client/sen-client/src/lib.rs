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
#![allow(dead_code)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::many_single_char_names, clippy::too_many_arguments))]

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

// use sen_core::sen_parse;
use sen_core::vm::{Vm, Env};
use sen_core::parser::*;
use sen_core::compiler::compile_program;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Bridge {
    vm: Vm,
    env: Env,

    source_buffer: String,
    out_source_buffer: String,
    traits_buffer: String,
    genotype_buffer: String,
}

#[wasm_bindgen]
impl Bridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bridge {
        Bridge {
            vm: Vm::new(),
            env: Env::new(),

            source_buffer: "source buffer".to_string(),
            out_source_buffer: "out_source buffer".to_string(),
            traits_buffer: "traits buffer".to_string(),
            genotype_buffer: "genotype buffer".to_string(),
        }
    }

    pub fn get_genotype_buffer_string(&self) -> String {
        log("get_genotype_buffer_string");
        self.genotype_buffer.to_string()
    }

    pub fn set_genotype_buffer_string(&mut self, s: &str) {
        log("set_genotype_buffer_string");
        self.genotype_buffer = s.to_string();
    }

    pub fn get_traits_buffer_string(&self) -> String {
        log("get_traits_buffer_string");
        self.traits_buffer.to_string()
    }

    pub fn set_traits_buffer_string(&mut self, s: &str) {
        log("set_traits_buffer_string");
        self.traits_buffer = s.to_string();
    }

    pub fn get_out_source_buffer_string(&self) -> String {
        log("get_out_source_buffer_string");
        self.out_source_buffer.to_string()
    }

    pub fn set_out_source_buffer_string(&mut self, s: &str) {
        log("set_out_source_buffer_string");
        self.out_source_buffer = s.to_string();
    }

    pub fn get_source_buffer_string(&self) -> String {
        log("get_source_buffer_string");
        self.source_buffer.to_string()
    }

    pub fn set_source_buffer_string(&mut self, s: &str) {
        log("set_source_buffer_string");
        self.source_buffer = s.to_string();
    }

    pub fn sen_startup(&self) {
        log("sen_startup");
    }

    pub fn sen_shutdown(&self) {
        log("sen_shutdown");
    }

    pub fn compile_to_render_packets(&mut self) -> i32 {
        let env = Env::new();

        let (ast, _word_lut) = if let Ok((ast_, _word_lut_)) = parse(&self.source_buffer) {
            (ast_, _word_lut_)
        } else {
            return 0
        };

        let program = if let Ok(program_) = compile_program(&ast) {
            program_
        } else {
            return 0
        };

        self.vm.reset();
        if self.vm.interpret(&env, &program).is_ok() {
            self.vm.geometry.get_num_render_packets() as i32
        } else {
            0
        }
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        self.vm.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        self.vm.get_render_packet_geo_ptr(packet_number)
    }

    pub fn build_traits(&self) -> i32 {
        log("build_traits");
        0
    }

    pub fn single_genotype_from_seed(&self, _seed: i32) -> i32 {
        log("single_genotype_from_seed");
        0
    }

    pub fn create_initial_generation(&self, _population_size: i32, _seed: i32) -> i32 {
        log("create_initial_generation");
        0
    }

    pub fn genotype_move_to_buffer(&self, _index: i32) {
        log("genotype_move_to_buffer");
    }

    pub fn script_cleanup(&self) {
        log("script_cleanup");
    }

    pub fn use_genotype_when_compiling(&self, use_genotype: bool) {
        if use_genotype {
            log("use_genotype_when_compiling : using");
        } else {
            log("use_genotype_when_compiling : not using genotype");
        }

    }

    pub fn next_generation_prepare(&self) {
        log("next_generation_prepare");
    }

    pub fn next_generation_add_genotype(&self) {
        log("next_generation_add_genotype");
    }

    pub fn next_generation_build(&self, _parent_size: i32, _population_size: i32, _mutation_rate: f32, _rng: i32) -> bool {
        log("next_generation_build");
        false
    }

    pub fn unparse_with_genotype(&self) {
        log("unparse_with_genotype");
    }

    pub fn simplify_script(&self) {
        log("simplify_script");
    }
}
