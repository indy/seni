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

use crate::colour::{Colour, ColourFormat};
use crate::ease::Easing;
use crate::error::Result;
use crate::geometry::quadratic;
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::prng;
use crate::render_list::RenderList;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    render_list: &mut RenderList,
    matrix: &Matrix,
    tessellation: usize,
    coords: &[f32; 8],
    stroke_tessellation: usize,
    stroke_noise: f32,
    stroke_line_width_start: f32,
    stroke_line_width_end: f32,
    colour: &Rgb,
    colour_volatility: f32,
    seed: f32,
    mapping: Easing,
    uvm: &UvMapping,
) -> Result<()> {
    let x1 = coords[0];
    let x2 = coords[2];
    let x3 = coords[4];
    let x4 = coords[6];
    let y1 = coords[1];
    let y2 = coords[3];
    let y3 = coords[5];
    let y4 = coords[7];

    let si_num = tessellation + 2;
    let si_unit = 1.0 / (si_num as f32 - 1.0);

    // todo: this lab colour manipulation is probably happening at the wrong level
    // everything within geometry should really only be using Rgb
    //
    let cc = Colour::new(ColourFormat::Rgb, colour.0, colour.1, colour.2, colour.3);
    let mut lab = cc.convert(ColourFormat::Lab)?;
    let lab_l = lab.e0;

    for i in 0..tessellation {
        let tvals0 = i as f32 * si_unit;
        let tvals1 = (i + 1) as f32 * si_unit;
        let tvals2 = (i + 2) as f32 * si_unit;

        // get 3 points on the bezier curve
        let xx1 = bezier_point(x1, x2, x3, x4, tvals0);
        let xx2 = bezier_point(x1, x2, x3, x4, tvals1);
        let xx3 = bezier_point(x1, x2, x3, x4, tvals2);

        let yy1 = bezier_point(y1, y2, y3, y4, tvals0);
        let yy2 = bezier_point(y1, y2, y3, y4, tvals1);
        let yy3 = bezier_point(y1, y2, y3, y4, tvals2);

        let ns = stroke_noise;

        lab.e0 = lab_l + (prng::perlin(xx1, xx1, xx1) * colour_volatility);

        let quad_coords: [f32; 6] = [
            xx1 + (ns * prng::perlin(xx1, xx1, seed)),
            yy1 + (ns * prng::perlin(yy1, yy1, seed)),
            xx2 + (ns * prng::perlin(xx2, xx1, seed)),
            yy2 + (ns * prng::perlin(yy2, yy1, seed)),
            xx3 + (ns * prng::perlin(xx3, xx1, seed)),
            yy3 + (ns * prng::perlin(yy3, yy1, seed)),
        ];

        let rgb_from_lab = Rgb::from_colour(&lab)?;
        quadratic::render(
            render_list,
            matrix,
            &quad_coords,
            stroke_line_width_start,
            stroke_line_width_end,
            mapping,
            0.0,
            1.0,
            &rgb_from_lab,
            stroke_tessellation,
            uvm,
        )?;
    }
    Ok(())
}
