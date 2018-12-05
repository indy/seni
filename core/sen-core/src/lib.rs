#![cfg_attr(
    feature = "cargo-clippy",
    allow(many_single_char_names, excessive_precision)
)]
#![allow(dead_code)]
// todo: remove crate wide allowing of dead_code

// Copyright (C) 2018 Inderjit Gill

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

mod colour;
mod compiler;
mod error;
mod lexer;
mod opcodes;
mod parser;
mod keywords;

pub use error::*;
pub use compiler::*;

pub fn sen_parse(s: &str) -> i32 {
    s.len() as i32
}
