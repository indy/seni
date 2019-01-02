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

#[wasm_bindgen]
pub fn lenlen() {
    log(&format!("hello has {} characters", sen_parse("hello")));
}
