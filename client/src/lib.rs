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
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::many_single_char_names, clippy::too_many_arguments)
)]

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use core::{
    build_traits, compile_to_render_packets, compile_with_genotype_to_render_packets,
    next_generation, simplified_unparse, unparse,
};
use core::{Env, Genotype, Packable, TraitList, Vm};

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

    genotype_list: Vec<Genotype>,

    use_genotype: bool,
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

            genotype_list: vec![],

            use_genotype: false,
        }
    }

    pub fn get_genotype_buffer_string(&self) -> String {
        // log("get_genotype_buffer_string");
        self.genotype_buffer.to_string()
    }

    pub fn set_genotype_buffer_string(&mut self, s: &str) {
        // log("set_genotype_buffer_string");
        self.genotype_buffer = s.to_string();
    }

    pub fn get_traits_buffer_string(&self) -> String {
        // log("get_traits_buffer_string");
        self.traits_buffer.to_string()
    }

    pub fn set_traits_buffer_string(&mut self, s: &str) {
        // log("set_traits_buffer_string");
        self.traits_buffer = s.to_string();
    }

    pub fn get_out_source_buffer_string(&self) -> String {
        // log("get_out_source_buffer_string");
        self.out_source_buffer.to_string()
    }

    pub fn set_out_source_buffer_string(&mut self, s: &str) {
        // log("set_out_source_buffer_string");
        self.out_source_buffer = s.to_string();
    }

    pub fn get_source_buffer_string(&self) -> String {
        // log("get_source_buffer_string");
        self.source_buffer.to_string()
    }

    pub fn set_source_buffer_string(&mut self, s: &str) {
        // log("set_source_buffer_string");
        self.source_buffer = s.to_string();
    }

    pub fn sen_startup(&self) {
        log("sen_startup");
    }

    pub fn sen_shutdown(&self) {
        // log("sen_shutdown");
    }

    pub fn compile_to_render_packets(&mut self) -> i32 {
        let mut num_render_packets = 0;

        if self.use_genotype {
            if let Ok((mut genotype, _)) = Genotype::unpack(&self.genotype_buffer) {
                num_render_packets = if let Ok(res) = compile_with_genotype_to_render_packets(
                    &mut self.vm,
                    &self.source_buffer,
                    &mut genotype,
                ) {
                    res
                } else {
                    0
                };
            } else {
                log("compile_to_render_packets: Genotype failed to unpack");
            }
        } else {
            num_render_packets =
                if let Ok(res) = compile_to_render_packets(&mut self.vm, &self.source_buffer) {
                    res
                } else {
                    0
                };
        }

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
        match build_traits(&self.source_buffer) {
            Ok(trait_list) => {
                self.traits_buffer = "".to_string();
                let packed_trait_list_res = trait_list.pack(&mut self.traits_buffer);

                return packed_trait_list_res.is_ok();
            }
            Err(e) => log(&format!("{:?}", e)),
        }

        false
    }

    pub fn single_genotype_from_seed(&mut self, seed: i32) -> bool {
        // log("single_genotype_from_seed");

        if let Ok((trait_list, _)) = TraitList::unpack(&self.traits_buffer) {
            if let Ok(genotype) = Genotype::build_from_seed(&trait_list, seed) {
                self.genotype_list = vec![genotype];
                return true;
            } else {
                log("single_genotype_from_seed: can't build genotype from unpacked TraitList");
                return false;
            }
        } else {
            log("single_genotype_from_seed: TraitList failed to unpack");
            return false;
        }
    }

    // todo: is bool the best return type?
    pub fn create_initial_generation(&mut self, population_size: i32, seed: i32) -> bool {
        if let Ok((trait_list, _)) = TraitList::unpack(&self.traits_buffer) {
            if let Ok(genotype_list) = Genotype::build_genotypes(&trait_list, population_size, seed)
            {
                self.genotype_list = genotype_list;
                return true;
            } else {
                log("create_initial_generation: can't build genotypes from unpacked TraitList");
                return false;
            }
        } else {
            log("create_initial_generation: TraitList failed to unpack");
            return false;
        }
    }

    pub fn genotype_move_to_buffer(&mut self, index: usize) -> bool {
        if let Some(ref genotype) = self.genotype_list.get(index) {
            self.genotype_buffer = "".to_string();
            let res = genotype.pack(&mut self.genotype_buffer);

            return res.is_ok();
        }

        false
    }

    pub fn script_cleanup(&self) {
        // log("script_cleanup");
    }

    pub fn use_genotype_when_compiling(&mut self, use_genotype: bool) {
        self.use_genotype = use_genotype;
    }

    pub fn next_generation_prepare(&mut self) {
        // log("next_generation_prepare");
        self.genotype_list = vec![];
    }

    pub fn next_generation_add_genotype(&mut self) {
        // log("next_generation_add_genotype");

        if let Ok((genotype, _)) = Genotype::unpack(&self.genotype_buffer) {
            self.genotype_list.push(genotype);
        } else {
            log("next_generation_add_genotype: Genotype failed to unpack");
        }
    }

    // todo: population_size should be usize
    pub fn next_generation_build(
        &mut self,
        parent_size: usize,
        population_size: i32,
        mutation_rate: f32,
        rng: i32,
    ) -> bool {
        // log("next_generation_build");

        // confirm that we have parent_size genotypes in self.genotype_list
        if self.genotype_list.len() != parent_size {
            log(&format!(
                "next_generation_build: parent_size ({}) mismatch with genotypes given ({})",
                parent_size,
                self.genotype_list.len()
            ));
            return false;
        }

        if let Ok((trait_list, _)) = TraitList::unpack(&self.traits_buffer) {
            match next_generation(
                &self.genotype_list,
                population_size as usize,
                mutation_rate,
                rng,
                &trait_list,
            ) {
                Ok(new_generation) => {
                    self.genotype_list = new_generation;
                    return true;
                }
                Err(e) => {
                    log(&format!("{:?}", e));
                    return false;
                }
            }
        } else {
            log("next_generation_build: TraitList failed to unpack");
            return false;
        }
    }

    pub fn unparse_with_genotype(&mut self) {
        if let Ok((mut genotype, _)) = Genotype::unpack(&self.genotype_buffer) {
            if let Ok(out_source) = unparse(&self.source_buffer, &mut genotype) {
                self.out_source_buffer = out_source;
            } else {
                log("unparse_with_genotype: unparse failed");
            }
        } else {
            log("unparse_with_genotype: Genotype failed to unpack");
        }
    }

    pub fn simplify_script(&mut self) {
        if let Ok(out_source) = simplified_unparse(&self.source_buffer) {
            self.out_source_buffer = out_source;
        } else {
            log("simplify_script: failed");
        }
    }
}
