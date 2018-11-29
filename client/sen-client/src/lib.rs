#![allow(dead_code)]
#![cfg_attr(feature = "cargo-clippy", allow(many_single_char_names, too_many_arguments))]

extern crate cfg_if;
extern crate sen_core;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

// use sokoban_core::audio::AudioConfig;
// use sokoban_core::config::Config;
// use sokoban_core::controller::{Controller, ControllerAction, ControllerButton};
// use sokoban_core::error::SokobanError;
// use sokoban_core::game::Game;
// use sokoban_core::geometry::Geometry;
// use sokoban_core::host::Host;

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
pub fn say_hi() {
    log("hello from wasm world!!!");
}
