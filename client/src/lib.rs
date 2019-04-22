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

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use core::{
    build_traits, compile_to_render_packets, compile_with_genotype_to_render_packets,
    next_generation, simplified_unparse, unparse,
};
use core::{Genotype, Packable, TraitList, Vm};

use log::{info, error};

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            use log::Level;
            console_log::init_with_level(Level::Warn).expect("error initializing log");
        }
    } else {
        fn init_log() {}
    }
}

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
#[derive(Default)]
pub struct Bridge {
    vm: Vm,

    source_buffer: String,
    out_source_buffer: String,
    traits_buffer: String,
    genotype_buffer: String,

    genotype_list: Vec<Genotype>,

    use_genotype: bool,
}

#[wasm_bindgen]
pub fn init_client_system() {
    init_log();
    info!("init_client_system: It works!");
}

#[wasm_bindgen]
impl Bridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bridge {
        Bridge {
            vm: Vm::new(),

            source_buffer: "source buffer".to_string(),
            out_source_buffer: "out_source buffer".to_string(),
            traits_buffer: "traits buffer".to_string(),
            genotype_buffer: "genotype buffer".to_string(),

            genotype_list: vec![],

            use_genotype: false,
        }
    }

    pub fn get_genotype_buffer_string(&self) -> String {
        info!("get_genotype_buffer_string");

        self.genotype_buffer.to_string()
    }

    pub fn set_genotype_buffer_string(&mut self, s: &str) {
        info!("set_genotype_buffer_string");

        self.genotype_buffer = s.to_string();
    }

    pub fn get_traits_buffer_string(&self) -> String {
        info!("get_traits_buffer_string");

        self.traits_buffer.to_string()
    }

    pub fn set_traits_buffer_string(&mut self, s: &str) {
        info!("set_traits_buffer_string");

        self.traits_buffer = s.to_string();
    }

    pub fn get_out_source_buffer_string(&self) -> String {
        info!("get_out_source_buffer_string");

        self.out_source_buffer.to_string()
    }

    pub fn set_out_source_buffer_string(&mut self, s: &str) {
        info!("set_out_source_buffer_string");

        self.out_source_buffer = s.to_string();
    }

    pub fn get_source_buffer_string(&self) -> String {
        info!("get_source_buffer_string");

        self.source_buffer.to_string()
    }

    pub fn set_source_buffer_string(&mut self, s: &str) {
        info!("set_source_buffer_string");

        self.source_buffer = s.to_string();
    }

    pub fn compile_to_render_packets(&mut self) -> i32 {
        info!("compile_to_render_packets");

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
                error!("compile_to_render_packets: Genotype failed to unpack");
            }
        } else {
            num_render_packets =
                if let Ok(res) = compile_to_render_packets(&mut self.vm, &self.source_buffer) {
                    res
                } else {
                    0
                };
        }

        // if !self.vm.debug_str.is_empty() {
        //     log(&self.vm.debug_str);
        // }
        self.vm.debug_str_clear();

        num_render_packets
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        info!("get_render_packet_geo_len");

        self.vm.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        info!("get_render_packet_geo_ptr");

        self.vm.get_render_packet_geo_ptr(packet_number)
    }

    // todo: is bool the best return type?
    pub fn build_traits(&mut self) -> bool {
        info!("build_traits");

        match build_traits(&self.source_buffer) {
            Ok(trait_list) => {
                self.traits_buffer = "".to_string();
                let packed_trait_list_res = trait_list.pack(&mut self.traits_buffer);

                return packed_trait_list_res.is_ok();
            }
            Err(e) => error!("{:?}", e),
        }

        false
    }

    pub fn single_genotype_from_seed(&mut self, seed: i32) -> bool {
        info!("single_genotype_from_seed");

        if let Ok((trait_list, _)) = TraitList::unpack(&self.traits_buffer) {
            if let Ok(genotype) = Genotype::build_from_seed(&trait_list, seed) {
                self.genotype_list = vec![genotype];
                return true;
            } else {
                error!("single_genotype_from_seed: can't build genotype from unpacked TraitList");
                return false;
            }
        } else {
            error!("single_genotype_from_seed: TraitList failed to unpack");
            return false;
        }
    }

    // todo: is bool the best return type?
    pub fn create_initial_generation(&mut self, population_size: i32, seed: i32) -> bool {
        info!("create_initial_generation");

        if let Ok((trait_list, _)) = TraitList::unpack(&self.traits_buffer) {
            if let Ok(genotype_list) = Genotype::build_genotypes(&trait_list, population_size, seed)
            {
                self.genotype_list = genotype_list;
                return true;
            } else {
                error!("create_initial_generation: can't build genotypes from unpacked TraitList");
                return false;
            }
        } else {
            error!("create_initial_generation: TraitList failed to unpack");
            return false;
        }
    }

    pub fn genotype_move_to_buffer(&mut self, index: usize) -> bool {
        info!("genotype_move_to_buffer");

        if let Some(ref genotype) = self.genotype_list.get(index) {
            self.genotype_buffer = "".to_string();
            let res = genotype.pack(&mut self.genotype_buffer);

            return res.is_ok();
        }

        false
    }

    pub fn script_cleanup(&self) {
        info!("script_cleanup");
    }

    pub fn use_genotype_when_compiling(&mut self, use_genotype: bool) {
        info!("use_genotype_when_compiling");

        self.use_genotype = use_genotype;
    }

    pub fn next_generation_prepare(&mut self) {
        info!("next_generation_prepare");

        self.genotype_list = vec![];
    }

    pub fn next_generation_add_genotype(&mut self) {
        info!("next_generation_add_genotype");

        if let Ok((genotype, _)) = Genotype::unpack(&self.genotype_buffer) {
            self.genotype_list.push(genotype);
        } else {
            error!("next_generation_add_genotype: Genotype failed to unpack");
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
        info!("next_generation_build");

        // confirm that we have parent_size genotypes in self.genotype_list
        if self.genotype_list.len() != parent_size {
            error!(
                "next_generation_build: parent_size ({}) mismatch with genotypes given ({})",
                parent_size,
                self.genotype_list.len()
            );
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
                    error!("{:?}", e);
                    return false;
                }
            }
        } else {
            error!("next_generation_build: TraitList failed to unpack");
            return false;
        }
    }

    pub fn unparse_with_genotype(&mut self) {
        info!("unparse_with_genotype");

        if let Ok((mut genotype, _)) = Genotype::unpack(&self.genotype_buffer) {
            if let Ok(out_source) = unparse(&self.source_buffer, &mut genotype) {
                self.out_source_buffer = out_source;
            } else {
                error!("unparse_with_genotype: unparse failed");
            }
        } else {
            error!("unparse_with_genotype: Genotype failed to unpack");
        }
    }

    pub fn simplify_script(&mut self) {
        info!("simplify_script");

        if let Ok(out_source) = simplified_unparse(&self.source_buffer) {
            self.out_source_buffer = out_source;
        } else {
            error!("simplify_script: failed");
        }
    }
}
