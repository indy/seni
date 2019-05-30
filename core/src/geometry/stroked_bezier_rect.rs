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

use crate::colour::{Colour, ColourFormat};
use crate::ease::Easing;
use crate::geometry::stroked_bezier;
use crate::geometry::Geometry;
use crate::matrix::Matrix;
use crate::prng;
use crate::result::Result;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    position: (f32, f32),
    width: f32,
    height: f32,
    volatility: f32,
    overlap: f32,
    iterations: f32,
    seed: i32,
    tessellation: usize,
    stroke_tessellation: usize,
    stroke_noise: f32,
    colour: &Rgb,
    colour_volatility: f32,
    uvm: &UvMapping,
) -> Result<()> {
    let x = position.0;
    let y = position.1;

    let x_start = x - (width / 2.0);
    let y_start = y - (height / 2.0);

    let th_width = width / 3.0;
    let th_height = height / 3.0;
    let vol = volatility;

    // todo: this lab colour manipulation is probably happening at the wrong level
    // everything within geometry should really only be using Rgb
    //
    let cc = Colour::new(ColourFormat::Rgb, colour.0, colour.1, colour.2, colour.3);
    let mut half_alpha_col = cc.convert(ColourFormat::Lab)?;
    half_alpha_col.e3 *= 0.5;
    let rgb_from_lab = Rgb::from_colour(&half_alpha_col)?;

    let mut prng = prng::PrngStateStruct::new(seed, 0.0, 1.0);

    let iiterations = iterations as i32;

    // sum of all strip thicknesses
    let sum_thicknesses = height + ((iterations - 1.0) * overlap);
    let stroke_thickness = sum_thicknesses / iterations;
    let stroke_half_thickness = stroke_thickness / 2.0;
    let stroke_offset_factor = (height - overlap) / iterations;

    // horizontal strokes
    //
    for i in 0..iiterations {
        let h = y_start + stroke_half_thickness + (i as f32 * stroke_offset_factor);

        let coords: [f32; 8] = [
            (prng.prng_f32_range(-1.0, 1.0) * vol) + x_start + (0.0 * th_width),
            h + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + x_start + (1.0 * th_width),
            h + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + x_start + (2.0 * th_width),
            h + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + x_start + (3.0 * th_width),
            h + (prng.prng_f32_range(-1.0, 1.0) * vol),
        ];

        stroked_bezier::render(
            geometry,
            matrix,
            tessellation,
            &coords,
            stroke_tessellation,
            stroke_noise,
            stroke_thickness,
            stroke_thickness,
            &rgb_from_lab,
            colour_volatility,
            prng.prng_f32(),
            Easing::Linear,
            uvm,
        )?;
    }

    let sum_thicknesses = width + ((iterations - 1.0) * overlap);
    let stroke_thickness = sum_thicknesses / iterations;
    let stroke_half_thickness = stroke_thickness / 2.0;
    let stroke_offset_factor = (width - overlap) / iterations;

    for i in 0..iiterations {
        let v = x_start + stroke_half_thickness + (i as f32 * stroke_offset_factor);

        let coords: [f32; 8] = [
            v + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + y_start + (0.0 * th_height),
            v + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + y_start + (1.0 * th_height),
            v + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + y_start + (2.0 * th_height),
            v + (prng.prng_f32_range(-1.0, 1.0) * vol),
            (prng.prng_f32_range(-1.0, 1.0) * vol) + y_start + (3.0 * th_height),
        ];

        stroked_bezier::render(
            geometry,
            matrix,
            tessellation,
            &coords,
            stroke_tessellation,
            stroke_noise,
            stroke_thickness,
            stroke_thickness,
            &rgb_from_lab,
            colour_volatility,
            prng.prng_f32(),
            Easing::Linear,
            uvm,
        )?;
    }
    Ok(())
}
