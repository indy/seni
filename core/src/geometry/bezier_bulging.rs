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

use crate::ease::Easing;
use crate::geometry::bezier;
use crate::geometry::Geometry;
use crate::matrix::Matrix;
use crate::result::Result;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    coords: &[f32; 8],
    line_width: f32,
    t_start: f32,
    t_end: f32,
    colour: &Rgb,
    tessellation: usize,
    uvm: &UvMapping,
) -> Result<()> {
    let t_mid = (t_start + t_end) / 2.0;
    let new_tess = tessellation >> 1;

    // thin_fat
    bezier::render(
        geometry,
        matrix,
        coords,
        0.0,
        line_width,
        Easing::SlowInOut,
        t_start,
        t_mid,
        colour,
        new_tess,
        uvm,
    )?;

    // fat_thin
    bezier::render(
        geometry,
        matrix,
        coords,
        line_width,
        0.0,
        Easing::SlowInOut,
        t_mid,
        t_end,
        colour,
        new_tess,
        uvm,
    )?;

    Ok(())
}
