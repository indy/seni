// Copyright (C) 2020 Inderjit Gill <email@indy.io>

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

use crate::ease::{easing, Easing};
use crate::error::{Error, Result};
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::render_list::RenderList;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    render_list: &mut RenderList,
    matrix: &Matrix,
    coords: &[f32; 6],
    width_start: f32,
    width_end: f32,
    width_mapping: Easing,
    t_start: f32,
    t_end: f32,
    colour: &Rgb,
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
    let y0 = coords[1];
    let y1 = coords[3];
    let y2 = coords[5];

    let unit = (t_end - t_start) / (tessellation as f32 - 1.0);

    let tex_t = 1.0 / tessellation as f32;

    let x_r = ((x1 - x0) - 0.5 * (x2 - x0)) / (0.5 * (0.5 - 1.0));
    let x_s = x2 - x0 - x_r;

    let y_r = ((y1 - y0) - 0.5 * (y2 - y0)) / (0.5 * (0.5 - 1.0));
    let y_s = y2 - y0 - y_r;

    // this chunk of code is just to calc the initial verts for prepare_to_add_triangle_strip
    // and to get the appropriate render packet
    //
    let t_val = t_start;
    let t_val_next = t_start + (1.0 * unit);
    let xs = (x_r * t_val * t_val) + (x_s * t_val) + x0;
    let ys = (y_r * t_val * t_val) + (y_s * t_val) + y0;
    let xs_next = (x_r * t_val_next * t_val_next) + (x_s * t_val_next) + x0;
    let ys_next = (y_r * t_val_next * t_val_next) + (y_s * t_val_next) + y0;
    let (n1x, n1y) = normal(xs, ys, xs_next, ys_next);
    let from_interp = (from_m * t_val) + from_c;
    let to_interp = easing(from_interp, width_mapping);
    let half_width = (to_m * to_interp) + to_c;
    let v1x = (n1x * half_width) + xs;
    let v1y = (n1y * half_width) + ys;
    render_list.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y)?;
    let rp = render_list
        .render_packets
        .last_mut()
        .ok_or(Error::Geometry)?;
    let rpg = rp.get_mut_render_packet_geometry()?;

    for i in 0..(tessellation - 1) {
        let t_val = t_start + (i as f32 * unit);
        let t_val_next = t_start + ((i + 1) as f32 * unit);

        let xs = (x_r * t_val * t_val) + (x_s * t_val) + x0;
        let ys = (y_r * t_val * t_val) + (y_s * t_val) + y0;
        let xs_next = (x_r * t_val_next * t_val_next) + (x_s * t_val_next) + x0;
        let ys_next = (y_r * t_val_next * t_val_next) + (y_s * t_val_next) + y0;

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
        rpg.add_vertex(matrix, v1x, v1y, &colour, u, v);
        let u = lerp(uv_t, au, cu);
        let v = lerp(uv_t, av, cv);
        rpg.add_vertex(matrix, v2x, v2y, &colour, u, v);
    }

    // final 2 vertices for the end point
    let i = tessellation - 2;

    let t_val = t_start + (i as f32 * unit);
    let t_val_next = t_start + ((i + 1) as f32 * unit);

    let xs = (x_r * t_val * t_val) + (x_s * t_val) + x0;
    let ys = (y_r * t_val * t_val) + (y_s * t_val) + y0;
    let xs_next = (x_r * t_val_next * t_val_next) + (x_s * t_val_next) + x0;
    let ys_next = (y_r * t_val_next * t_val_next) + (y_s * t_val_next) + y0;

    let (n1x, n1y) = normal(xs, ys, xs_next, ys_next);
    let (n2x, n2y) = opposite_normal(n1x, n1y);

    let v1x = (n1x * half_width_end) + xs_next;
    let v1y = (n1y * half_width_end) + ys_next;
    let v2x = (n2x * half_width_end) + xs_next;
    let v2y = (n2y * half_width_end) + ys_next;

    rpg.add_vertex(matrix, v1x, v1y, &colour, du, dv);

    rpg.add_vertex(matrix, v2x, v2y, &colour, cu, cv);

    Ok(())
}
