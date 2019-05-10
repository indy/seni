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
use crate::ease::{easing, Easing};
use crate::geometry::Geometry;
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::result::Result;
use crate::uvmapper::UvMapping;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    coords: &[f32; 8],
    width_start: f32,
    width_end: f32,
    width_mapping: Easing,
    t_start: f32,
    t_end: f32,
    colour: &Colour,
    tessellation: usize,
    uvm: &UvMapping,
) -> Result<()> {
    let au = uvm.map[0];
    let av = uvm.map[1];
    let bu = uvm.map[2];
    let bv = uvm.map[3];
    let cu = uvm.map[4];
    let cv = uvm.map[5];
    let du = uvm.map[6];
    let dv = uvm.map[7];

    // modify the width so that the brush textures provide good coverage
    //
    let line_width_start = width_start * uvm.width_scale;
    let line_width_end = width_end * uvm.width_scale;

    // variables for interpolating the curve's width
    //
    let half_width_start = line_width_start / 2.0;
    let half_width_end = line_width_end / 2.0;
    let from_m = mc_m(t_start, 0.0, t_end, 1.0);
    let from_c = mc_c(t_start, 0.0, from_m);
    let to_m = mc_m(0.0, half_width_start, 1.0, half_width_end);
    let to_c = mc_c(0.0, half_width_start, to_m);

    let x0 = coords[0];
    let x1 = coords[2];
    let x2 = coords[4];
    let x3 = coords[6];
    let y0 = coords[1];
    let y1 = coords[3];
    let y2 = coords[5];
    let y3 = coords[7];

    let unit = (t_end - t_start) / (tessellation as f32 - 1.0);

    let tex_t = 1.0 / tessellation as f32;

    let rgb = colour.convert(ColourFormat::Rgb)?;

    // this chunk of code is just to calc the initial verts for prepare_to_add_triangle_strip
    // and to get the appropriate render packet
    //
    let t_val = t_start;
    let t_val_next = t_start + (1.0 * unit);
    let xs = bezier_point(x0, x1, x2, x3, t_val);
    let ys = bezier_point(y0, y1, y2, y3, t_val);
    let xs_next = bezier_point(x0, x1, x2, x3, t_val_next);
    let ys_next = bezier_point(y0, y1, y2, y3, t_val_next);
    let (n1x, n1y) = normal(xs, ys, xs_next, ys_next);
    let from_interp = (from_m * t_val) + from_c;
    let to_interp = easing(from_interp, width_mapping);
    let half_width = (to_m * to_interp) + to_c;
    let v1x = (n1x * half_width) + xs;
    let v1y = (n1y * half_width) + ys;
    geometry.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y);
    let last = geometry.render_packets.len() - 1;
    let rp = &mut geometry.render_packets[last];

    for i in 0..(tessellation - 1) {
        let t_val = t_start + (i as f32 * unit);
        let t_val_next = t_start + ((i + 1) as f32 * unit);

        let xs = bezier_point(x0, x1, x2, x3, t_val);
        let ys = bezier_point(y0, y1, y2, y3, t_val);
        let xs_next = bezier_point(x0, x1, x2, x3, t_val_next);
        let ys_next = bezier_point(y0, y1, y2, y3, t_val_next);

        // addVerticesAsStrip
        let (n1x, n1y) = normal(xs, ys, xs_next, ys_next);
        let (n2x, n2y) = opposite_normal(n1x, n1y);

        let from_interp = (from_m * t_val) + from_c;
        let to_interp = easing(from_interp, width_mapping);

        let half_width = (to_m * to_interp) + to_c;

        let v1x = (n1x * half_width) + xs;
        let v1y = (n1y * half_width) + ys;
        let v2x = (n2x * half_width) + xs;
        let v2y = (n2y * half_width) + ys;

        let uv_t = tex_t * (i as f32);
        let u = lerp(uv_t, bu, du);
        let v = lerp(uv_t, bv, dv);
        rp.add_vertex(matrix, v1x, v1y, &rgb, u, v);
        let u = lerp(uv_t, au, cu);
        let v = lerp(uv_t, av, cv);
        rp.add_vertex(matrix, v2x, v2y, &rgb, u, v);
    }

    // final 2 vertices for the end point
    let i = tessellation - 2;

    let t_val = t_start + (i as f32 * unit);
    let t_val_next = t_start + ((i + 1) as f32 * unit);

    let xs = bezier_point(x0, x1, x2, x3, t_val);
    let ys = bezier_point(y0, y1, y2, y3, t_val);
    let xs_next = bezier_point(x0, x1, x2, x3, t_val_next);
    let ys_next = bezier_point(y0, y1, y2, y3, t_val_next);

    let (n1x, n1y) = normal(xs, ys, xs_next, ys_next);
    let (n2x, n2y) = opposite_normal(n1x, n1y);

    let v1x = (n1x * half_width_end) + xs_next;
    let v1y = (n1y * half_width_end) + ys_next;
    let v2x = (n2x * half_width_end) + xs_next;
    let v2y = (n2y * half_width_end) + ys_next;

    rp.add_vertex(matrix, v1x, v1y, &rgb, du, dv);

    rp.add_vertex(matrix, v2x, v2y, &rgb, cu, cv);

    Ok(())
}