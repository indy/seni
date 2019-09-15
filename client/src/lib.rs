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

#![allow(dead_code)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::many_single_char_names, clippy::too_many_arguments)
)]

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use core::{
    build_traits, next_generation, program_from_source, program_from_source_and_genotype,
    run_program_with_preamble, simplified_unparse, unparse, bitmaps_to_transfer, textures_to_load
};
use core::{BitmapInfo, Context, Genotype, Packable, Program, TraitList, Vm};

use log::{error, info};

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
            console_log::init_with_level(Level::Error).expect("error initializing log");
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
    context: Context,

    genotype_list: Vec<Genotype>,
    // only used during sequence of calls for rendering
    program: Option<Program>,
}

#[wasm_bindgen]
pub fn init_client_system() {
    init_log();
    // info!("init_client_system");
}

#[wasm_bindgen]
impl Bridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Bridge {
        Bridge {
            vm: Default::default(),
            context: Default::default(),

            genotype_list: vec![],
            program: None,
        }
    }

    // --------------------------------------------------------------------------------
    // new rendering api
    pub fn compile_program_from_source(&mut self, source: &str) -> bool {
        let res = program_from_source(&source);
        match res {
            Ok(program) => {
                self.program = Some(program);
                true
            }
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }
    pub fn compile_program_from_source_and_genotype(
        &mut self,
        source: &str,
        packed_genotype: &str,
    ) -> bool {
        if let Ok((mut genotype, _)) = Genotype::unpack(packed_genotype) {
            let res = program_from_source_and_genotype(source, &mut genotype);
            match res {
                Ok(program) => {
                    self.program = Some(program);
                    true
                }
                Err(e) => {
                    error!("{}", e);
                    false
                }
            }
        } else {
            error!("program_from_source_and_genotype: Genotype failed to unpack");
            false
        }
    }

    // return the list of required bitmaps to the host system
    // it will send the bitmap data back here via the add_rgba_bitmap function
    pub fn get_bitmap_transfers_as_json(&self) -> String {
        // todo: check that we're in the RENDER state

        if let Some(program) = &self.program {
            let bitmaps_to_transfer = bitmaps_to_transfer(program, &self.context);
            vec_strings_as_json(&bitmaps_to_transfer)
        } else {
            "[]".to_string()
        }
    }

    // return the list of textures to load onto the GPU
    pub fn get_textures_to_load_as_json(&self) -> String {
        if let Some(program) = &self.program {
            let textures_to_load = textures_to_load(program, &self.context);
            vec_strings_as_json(&textures_to_load)
        } else {
            "[]".to_string()
        }
    }

    // the program has been compiled,
    // all required bitmaps have been loaded
    pub fn run_program(&mut self) -> usize {
        // todo: check that we're in the RENDER state

        if let Some(program) = &self.program {
            match run_program_with_preamble(&mut self.vm, &mut self.context, &program) {
                Ok(_) => {
                    self.vm.debug_str_clear();
                    self.context.geometry.remove_useless_render_packets();
                    self.context.geometry.get_num_render_packets()
                }
                Err(e) => {
                    error!("{}", e);
                    0
                }
            }
        } else {
            0
        }
    }

    pub fn output_linear_colour_space(&self) -> bool {
        self.context.output_linear_colour_space
    }

    // --------------------------------------------------------------------------------

    pub fn get_render_packet_command(&self, packet_number: usize) -> i32 {
        self.context.get_render_packet_command(packet_number)
    }

    pub fn get_render_packet_mask_filename(&self, packet_number: usize) -> String {
        self.context.get_render_packet_mask_filename(packet_number)
    }

    pub fn get_render_packet_mask_invert(&self, packet_number: usize) -> bool {
        self.context.get_render_packet_mask_invert(packet_number)
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        // info!("get_render_packet_geo_len");

        self.context.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        // info!("get_render_packet_geo_ptr");

        self.context.get_render_packet_geo_ptr(packet_number)
    }

    pub fn add_rgba_bitmap(&mut self, name: &str, width: usize, height: usize, data: Vec<u8>) {
        let bitmap_info = BitmapInfo::new(width, height, data);

        self.context.bitmap_cache.insert(name, bitmap_info).unwrap();
    }

    // todo: is bool the best return type?
    pub fn build_traits(&mut self, source: &str) -> String {
        info!("build_traits");

        match build_traits(source) {
            Ok(trait_list) => {
                let mut traits_buffer = "".to_string();
                let packed_trait_list_res = trait_list.pack(&mut traits_buffer);

                if packed_trait_list_res.is_ok() {
                    return traits_buffer;
                }
            }
            Err(e) => error!("{:?}", e),
        }

        "".to_string()
    }

    pub fn single_genotype_from_seed(&mut self, packed_trait_list: &str, seed: i32) -> bool {
        info!("single_genotype_from_seed");

        if let Ok((trait_list, _)) = TraitList::unpack(packed_trait_list) {
            if let Ok(genotype) = Genotype::build_from_seed(&trait_list, seed) {
                self.genotype_list = vec![genotype];
                true
            } else {
                error!("single_genotype_from_seed: can't build genotype from unpacked TraitList");
                false
            }
        } else {
            error!("single_genotype_from_seed: TraitList failed to unpack");
            false
        }
    }

    // todo: is bool the best return type?
    pub fn create_initial_generation(
        &mut self,
        packed_trait_list: &str,
        population_size: i32,
        seed: i32,
    ) -> bool {
        info!("create_initial_generation");

        if let Ok((trait_list, _)) = TraitList::unpack(packed_trait_list) {
            if let Ok(genotype_list) = Genotype::build_genotypes(&trait_list, population_size, seed)
            {
                self.genotype_list = genotype_list;
                true
            } else {
                error!("create_initial_generation: can't build genotypes from unpacked TraitList");
                false
            }
        } else {
            error!("create_initial_generation: TraitList failed to unpack");
            false
        }
    }

    pub fn get_genotype(&mut self, index: usize) -> String {
        info!("genotype_move_to_buffer");

        if let Some(ref genotype) = self.genotype_list.get(index) {
            let mut packed_genotype = "".to_string();
            let res = genotype.pack(&mut packed_genotype);

            if res.is_ok() {
                return packed_genotype;
            }
        }

        "".to_string()
    }

    pub fn script_cleanup(&self) {
        info!("script_cleanup");
    }

    pub fn next_generation_prepare(&mut self) {
        info!("next_generation_prepare");

        self.genotype_list = vec![];
    }

    pub fn next_generation_add_genotype(&mut self, packed_genotype: &str) {
        info!("next_generation_add_genotype");

        if let Ok((genotype, _)) = Genotype::unpack(packed_genotype) {
            self.genotype_list.push(genotype);
        } else {
            error!("next_generation_add_genotype: Genotype failed to unpack");
        }
    }

    // todo: population_size should be usize
    pub fn next_generation_build(
        &mut self,
        packed_trait_list: &str,
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

        if let Ok((trait_list, _)) = TraitList::unpack(packed_trait_list) {
            match next_generation(
                &self.genotype_list,
                population_size as usize,
                mutation_rate,
                rng,
                &trait_list,
            ) {
                Ok(new_generation) => {
                    self.genotype_list = new_generation;
                    true
                }
                Err(e) => {
                    error!("{:?}", e);
                    false
                }
            }
        } else {
            error!("next_generation_build: TraitList failed to unpack");
            false
        }
    }

    pub fn unparse_with_genotype(&mut self, source: &str, packed_genotype: &str) -> String {
        info!("unparse_with_genotype");

        if let Ok((mut genotype, _)) = Genotype::unpack(packed_genotype) {
            if let Ok(out_source) = unparse(source, &mut genotype) {
                return out_source;
            } else {
                error!("unparse_with_genotype: unparse failed");
            }
        } else {
            error!("unparse_with_genotype: Genotype failed to unpack");
        }
        "".to_string()
    }

    pub fn simplify_script(&mut self, source: &str) -> String {
        info!("simplify_script");

        if let Ok(out_source) = simplified_unparse(source) {
            return out_source;
        } else {
            error!("simplify_script: failed");
        }
        "".to_string()
    }
}

fn vec_strings_as_json(strings: &Vec<String>) -> String {
    let mut res: String = "[".to_string();
    for (i, s) in strings.iter().enumerate() {
        res.push_str(&format!("\"{}\"", s));
        if i < strings.len() - 1 {
            res.push_str(", ");
        }
    }
    res.push_str("]");

    res
}
