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
    bitmaps_to_transfer, build_traits, next_generation, program_from_source,
    program_from_source_and_genotype, run_program_with_preamble, simplified_unparse, unparse,
};
use core::{
    BitmapInfo, Context, Genotype, Packable, Program, RenderPacketGeometry, RenderPacketImage,
    RenderPacketMask, TraitList, Vm,
};

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
pub struct RenderPacketGeometryWasm {
    geo_len: usize,
    geo_ptr: *const f32,
}

#[wasm_bindgen]
impl RenderPacketGeometryWasm {
    pub fn get_geo_len(&self) -> usize {
        self.geo_len
    }

    pub fn get_geo_ptr(&self) -> *const f32 {
        self.geo_ptr
    }
}

impl From<&RenderPacketGeometry> for RenderPacketGeometryWasm {
    fn from(rp: &RenderPacketGeometry) -> RenderPacketGeometryWasm {
        RenderPacketGeometryWasm {
            geo_len: rp.get_geo_len(),
            geo_ptr: rp.get_geo_ptr(),
        }
    }
}

// RenderPacketMaskWasm is a structure that mirrors RenderPacketMask.
// The only difference is that it is annotated with wasm_bindgen so
// it can be transferred over to the JS side
//
#[wasm_bindgen]
pub struct RenderPacketMaskWasm {
    filename: String,
    invert: bool,
}

#[wasm_bindgen]
impl RenderPacketMaskWasm {
    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }
    pub fn get_invert(&self) -> bool {
        self.invert
    }
}

impl From<&RenderPacketMask> for RenderPacketMaskWasm {
    fn from(rpm: &RenderPacketMask) -> RenderPacketMaskWasm {
        RenderPacketMaskWasm {
            filename: rpm.filename.clone(),
            invert: rpm.invert,
        }
    }
}

#[wasm_bindgen]
pub struct RenderPacketImageWasm {
    linear_colour_space: bool,
    contrast: f32,
    brightness: f32,
    saturation: f32,
}

#[wasm_bindgen]
impl RenderPacketImageWasm {
    pub fn get_linear_colour_space(&self) -> bool {
        self.linear_colour_space
    }
    pub fn get_contrast(&self) -> f32 {
        self.contrast
    }
    pub fn get_brightness(&self) -> f32 {
        self.brightness
    }
    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }
}

impl From<&RenderPacketImage> for RenderPacketImageWasm {
    fn from(rpi: &RenderPacketImage) -> RenderPacketImageWasm {
        RenderPacketImageWasm {
            linear_colour_space: rpi.linear_colour_space,
            contrast: rpi.contrast,
            brightness: rpi.brightness,
            saturation: rpi.saturation,
        }
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

    // the program has been compiled,
    // all required bitmaps have been loaded
    pub fn run_program(&mut self) -> usize {
        // todo: check that we're in the RENDER state

        if let Some(program) = &self.program {
            match run_program_with_preamble(&mut self.vm, &mut self.context, &program) {
                Ok(_) => {
                    self.vm.debug_str_clear();
                    self.context.render_list.remove_useless_render_packets();
                    self.context.render_list.get_num_render_packets()
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

    // --------------------------------------------------------------------------------

    pub fn rp_command(&self, packet_number: usize) -> i32 {
        let command = self
            .context
            .get_render_packet_command(packet_number)
            .unwrap();

        command as i32
    }

    pub fn rp_mask(&self, packet_number: usize) -> RenderPacketMaskWasm {
        let rpm = self.context.get_render_packet_mask(packet_number).unwrap();
        RenderPacketMaskWasm::from(rpm)
    }

    pub fn rp_image(&self, packet_number: usize) -> RenderPacketImageWasm {
        let rpi = self.context.get_render_packet_image(packet_number).unwrap();
        RenderPacketImageWasm::from(rpi)
    }

    pub fn rp_geometry(&self, packet_number: usize) -> RenderPacketGeometryWasm {
        let geometry = self
            .context
            .get_render_packet_geometry(packet_number)
            .unwrap();
        RenderPacketGeometryWasm::from(geometry)
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
