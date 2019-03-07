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

use sen_core;

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
    vm: sen_core::Vm,
    env: sen_core::Env,

    source_buffer: String,
    out_source_buffer: String,
    traits_buffer: String,
    genotype_buffer: String,

    genotype_list: Option<Vec<sen_core::Genotype>>,

    // temporary, these will all be removed eventually
    trait_list: Option<sen_core::TraitList>,
}

#[wasm_bindgen]
impl Bridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bridge {
        Bridge {
            vm: sen_core::Vm::new(),
            env: sen_core::Env::new(),

            source_buffer: "source buffer".to_string(),
            out_source_buffer: "out_source buffer".to_string(),
            traits_buffer: "traits buffer".to_string(),
            genotype_buffer: "genotype buffer".to_string(),

            genotype_list: None,

            trait_list: None,
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
        let num_render_packets = if let Ok(res) = sen_core::compile_to_render_packets(&mut self.vm, &self.source_buffer) {
            res
        } else {
            0
        };

        log(&self.vm.debug_str);
        self.vm.debug_str_clear();

        num_render_packets
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        self.vm.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        self.vm.get_render_packet_geo_ptr(packet_number)
    }

    // todo: is bool the best return type?
    pub fn build_traits(&mut self) -> bool {
        if let Ok(trait_list) = sen_core::build_traits(&self.source_buffer) {
            self.trait_list = Some(trait_list);
            // todo: serialize trait_list
            true
        } else {
            false
        }
    }

    pub fn single_genotype_from_seed(&mut self, seed: i32) -> bool {
        // todo: deserialize trait_list from traits_buffer
        // for the moment using self.trait_list THIS IS WRONG!!!
        if let Some(ref trait_list) = self.trait_list {
            if let Ok(genotype) = sen_core::Genotype::build_from_seed(trait_list, seed) {
                self.genotype_list = Some(vec![genotype]);
                return true
            }
        }
        false
    }

    // todo: is bool the best return type?
    pub fn create_initial_generation(&mut self, population_size: i32, seed: i32) -> bool {
        // todo: deserialize trait_list from traits_buffer
        // for the moment using self.trait_list THIS IS WRONG!!!
        if let Some(ref trait_list) = self.trait_list {
            if let Ok(genotype_list) = sen_core::Genotype::build_genotypes(&trait_list, population_size, seed) {
                self.genotype_list = Some(genotype_list);
                return true
            }
        }

        false
    }

    pub fn genotype_move_to_buffer(&mut self, index: usize) -> bool {
        if let Some(ref genotype_list) = self.genotype_list {
            if let Some(ref _genotype) = genotype_list.get(index) {
                // todo: serialize into genotype_buffer
                unimplemented!();
                // return true
            }
        }
        false
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
