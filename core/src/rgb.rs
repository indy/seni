// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::colour::{Colour, ColourFormat};
use crate::error::Result;

// Used by low level modules that expect RGB colours, more restrictive than
// Colour since a couple of bugs were caused by passing in non-RGB Colour
// structs into functions expecting Colour in RGB format.
//
#[derive(Copy, Clone, Debug)]
pub struct Rgb(pub f32, pub f32, pub f32, pub f32);

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Rgb(r, g, b, alpha)
    }

    pub fn from_colour(col: &Colour) -> Result<Rgb> {
        let rgb = col.convert(ColourFormat::Rgb)?;
        Ok(Rgb(rgb.e0, rgb.e1, rgb.e2, rgb.e3))
    }
}
