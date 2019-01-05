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
#![cfg_attr(feature = "cargo-clippy", allow(many_single_char_names, too_many_arguments))]

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use sen_core::sen_parse;
use sen_core::geometry::Geometry;

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
pub struct Bridge {
    geometry: Geometry,

    source_buffer: String,
    out_source_buffer: String,
    traits_buffer: String,
    genotype_buffer: String,
}

#[wasm_bindgen]
impl Bridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bridge {
        let geometry = Geometry::new();

        Bridge {
            geometry,
            source_buffer: "source buffer".to_string(),
            out_source_buffer: "out_source buffer".to_string(),
            traits_buffer: "traits buffer".to_string(),
            genotype_buffer: "genotype buffer".to_string(),
        }
    }

    pub fn say_hi(&self) {
        log("bridge: hello from wasm world!!!");
    }

    pub fn lenlen(&self) {
        log(&format!("hello has {} characters", sen_parse("hello")));
    }

    pub fn get_genotype_buffer_string(&self) -> String {
        self.genotype_buffer.to_string()
    }

    pub fn set_genotype_buffer_string(&mut self, s: &str) {
        self.genotype_buffer = s.to_string();
    }

    pub fn get_traits_buffer_string(&self) -> String {
        self.traits_buffer.to_string()
    }

    pub fn set_traits_buffer_string(&mut self, s: &str) {
        self.traits_buffer = s.to_string();
    }

    pub fn get_out_source_buffer_string(&self) -> String {
        self.out_source_buffer.to_string()
    }

    pub fn set_out_source_buffer_string(&mut self, s: &str) {
        self.out_source_buffer = s.to_string();
    }

    pub fn get_source_buffer_string(&self) -> String {
        self.source_buffer.to_string()
    }

    pub fn set_source_buffer_string(&mut self, s: &str) {
        self.source_buffer = s.to_string();
    }

    pub fn sen_startup(&self) {
        log("sen_startup");
    }

    pub fn sen_shutdown(&self) {
        log("sen_shutdown");
    }

    pub fn compile_to_render_packets(&self) -> i32 {
        log("compile_to_render_packets");
        0
    }

    pub fn get_render_packet_num_vertices(&self, _packet_number: i32) -> i32 {
        log("get_render_packet_num_vertices");
        0
    }

    pub fn get_render_packet_vbuf(&self, _packet_number: i32) -> *const f32 {
        // should be a pointer to f32 ???
        0 as *const f32
    }

    pub fn get_render_packet_cbuf(&self, _packet_number: i32) -> *const f32 {
        // should be a pointer to f32 ???
        0 as *const f32
    }

    pub fn get_render_packet_tbuf(&self, _packet_number: i32) -> *const f32 {
        // should be a pointer to f32 ???
        0 as *const f32
    }

    pub fn build_traits(&self) -> i32 {
        0
    }

    pub fn single_genotype_from_seed(&self, _seed: i32) -> i32 {
        0
    }

    pub fn create_initial_generation(&self, _population_size: i32, _seed: i32) -> i32 {
        0
    }

    pub fn genotype_move_to_buffer(&self, _index: i32) {}

    pub fn script_cleanup(&self) {
        log("script_cleanup");
    }

    pub fn use_genotype_when_compiling(&self, _use_genotype: bool) {}

    pub fn next_generation_prepare(&self) {}

    pub fn next_generation_add_genotype(&self) {}

    pub fn next_generation_build(&self, _parent_size: i32, _population_size: i32, _mutation_rate: f32, _rng: i32) -> bool {
        false
    }

    pub fn unparse_with_genotype(&self) {}

    pub fn simplify_script(&self) {}
}
