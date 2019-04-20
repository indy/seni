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
use crate::error::*;
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::prng;
use crate::uvmapper::UvMapping;
use crate::vm::Var;

// todo: work out reasonable defaults
const RENDER_PACKET_MAX_SIZE: usize = 40096;
pub const RENDER_PACKET_FLOAT_PER_VERTEX: usize = 8;

#[derive(Default)]
pub struct RenderPacket {
    pub geo: Vec<f32>,
}

#[derive(Default)]
pub struct Geometry {
    render_packets: Vec<RenderPacket>,
}

impl RenderPacket {
    pub fn new() -> Self {
        RenderPacket {
            geo: Vec::with_capacity(RENDER_PACKET_MAX_SIZE),
        }
    }

    pub fn get_geo_len(&self) -> usize {
        self.geo.len()
    }

    pub fn get_geo_ptr(&self) -> *const f32 {
        self.geo.as_ptr() as *const f32
    }

    pub fn add_vertex(&mut self, matrix: &Matrix, x: f32, y: f32, col: &Colour, u: f32, v: f32) {
        // assuming that col is ColourFormat::Rgb

        let (nx, ny) = matrix.transform_vec2(x, y);
        // pre-multiply the alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        self.geo.append(&mut vec![
            nx,
            ny,
            col.e0 * col.e3,
            col.e1 * col.e3,
            col.e2 * col.e3,
            col.e3,
            u,
            v,
        ]);
    }

    pub fn form_degenerate_triangle(&mut self, matrix: &Matrix, x: f32, y: f32) {
        // just copy the previous entries
        self.dup();

        // add the new vertex to complete the degenerate triangle
        self.add_vertex(matrix, x, y, &Colour::default(), 0.0, 0.0);

        // Note: still need to call addVertex on the first
        // vertex when we 'really' render the strip
    }

    // duplicate the last geometry point
    pub fn dup(&mut self) {
        let len = self.geo.len();

        let x: f32;
        let y: f32;
        let r: f32;
        let g: f32;
        let b: f32;
        let a: f32;
        let u: f32;
        let v: f32;
        {
            x = self.geo[len - 8];
            y = self.geo[len - 7];
            r = self.geo[len - 6];
            g = self.geo[len - 5];
            b = self.geo[len - 4];
            a = self.geo[len - 3];
            u = self.geo[len - 2];
            v = self.geo[len - 1];
        }

        self.geo.append(&mut vec![x, y, r, g, b, a, u, v]);
    }

    pub fn can_vertices_fit(&self, num: usize) -> bool {
        self.geo.len() < (RENDER_PACKET_MAX_SIZE - (num * RENDER_PACKET_FLOAT_PER_VERTEX))
    }

    pub fn is_empty(&self) -> bool {
        self.geo.is_empty()
    }
}

impl Geometry {
    pub fn new() -> Geometry {
        let mut render_packets: Vec<RenderPacket> = Vec::new();
        render_packets.push(RenderPacket::new());

        Geometry { render_packets }
    }

    pub fn reset(&mut self) {
        self.render_packets.clear();
        self.render_packets.push(RenderPacket::new())
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        let rp = &self.render_packets[packet_number];
        rp.geo.len()
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        let rp = &self.render_packets[packet_number];
        rp.geo.as_ptr() as *const f32
    }

    pub fn get_num_render_packets(&self) -> usize {
        self.render_packets.len()
    }

    fn prepare_to_add_triangle_strip(
        &mut self,
        matrix: &Matrix,
        num_vertices: usize,
        x: f32,
        y: f32,
    ) {
        let mut last = self.render_packets.len() - 1;
        let mut rp = &mut self.render_packets[last];
        if !rp.can_vertices_fit(num_vertices) {
            self.render_packets.push(RenderPacket::new());
            last += 1;
        }

        rp = &mut self.render_packets[last];
        if !rp.is_empty() {
            rp.form_degenerate_triangle(matrix, x, y);
        }
    }

    pub fn render_line(
        &mut self,
        matrix: &Matrix,
        from: (f32, f32),
        to: (f32, f32),
        width: f32,
        from_col: &Colour,
        to_col: &Colour,
        uvm: &UvMapping,
    ) -> Result<()> {
        let hw = (width * uvm.width_scale) / 2.0;

        let (nx, ny) = normal(from.0, from.1, to.0, to.1);
        let (n2x, n2y) = opposite_normal(nx, ny);

        self.prepare_to_add_triangle_strip(matrix, 4, from.0 + (hw * nx), from.1 + (hw * ny));

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        rp.add_vertex(
            matrix,
            from.0 + (hw * nx),
            from.1 + (hw * ny),
            from_col,
            uvm.map[0],
            uvm.map[1],
        );
        rp.add_vertex(
            matrix,
            from.0 + (hw * n2x),
            from.1 + (hw * n2y),
            from_col,
            uvm.map[2],
            uvm.map[3],
        );
        rp.add_vertex(
            matrix,
            to.0 + (hw * nx),
            to.1 + (hw * ny),
            to_col,
            uvm.map[4],
            uvm.map[5],
        );
        rp.add_vertex(
            matrix,
            to.0 + (hw * n2x),
            to.1 + (hw * n2y),
            to_col,
            uvm.map[6],
            uvm.map[7],
        );

        Ok(())
    }

    pub fn render_rect(
        &mut self,
        matrix: &Matrix,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        uvm: &UvMapping,
    ) -> Result<()> {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        self.prepare_to_add_triangle_strip(
            matrix,
            4,
            position.0 - half_width,
            position.1 - half_height,
        );

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        rp.add_vertex(
            matrix,
            position.0 - half_width,
            position.1 - half_height,
            colour,
            uvm.map[0],
            uvm.map[1],
        );
        rp.add_vertex(
            matrix,
            position.0 + half_width,
            position.1 - half_height,
            colour,
            uvm.map[2],
            uvm.map[3],
        );
        rp.add_vertex(
            matrix,
            position.0 - half_width,
            position.1 + half_height,
            colour,
            uvm.map[4],
            uvm.map[5],
        );
        rp.add_vertex(
            matrix,
            position.0 + half_width,
            position.1 + half_height,
            colour,
            uvm.map[6],
            uvm.map[7],
        );

        Ok(())
    }

    pub fn render_circle(
        &mut self,
        matrix: &Matrix,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
        uvm: &UvMapping,
    ) -> Result<()> {
        self.prepare_to_add_triangle_strip(matrix, (tessellation * 2) + 2, position.0, position.1);

        let unit_angle = TAU / tessellation as f32;

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        for i in 0..tessellation {
            let angle = unit_angle * i as f32;
            let vx = (angle.sin() * width) + position.0;
            let vy = (angle.cos() * height) + position.1;

            rp.add_vertex(
                matrix, position.0, position.1, colour, uvm.map[4], uvm.map[5],
            );
            rp.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);
        }

        let angle: f32 = 0.0;
        let vx = (angle.sin() * width) + position.0;
        let vy = (angle.cos() * height) + position.1;

        rp.add_vertex(
            matrix, position.0, position.1, colour, uvm.map[4], uvm.map[5],
        );
        rp.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);

        Ok(())
    }

    pub fn render_circle_slice(
        &mut self,
        matrix: &Matrix,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
        angle_start: f32,
        angle_end: f32,
        inner_width: f32,
        inner_height: f32,
        uvm: &UvMapping,
    ) -> Result<()> {
        let r_start = deg_to_rad(angle_start);
        let r_end = deg_to_rad(angle_end);
        let unit_angle = (r_end - r_start) / tessellation as f32;

        let mut innervx = (r_start.sin() * inner_width) + position.0;
        let mut innervy = (r_start.cos() * inner_height) + position.1;

        self.prepare_to_add_triangle_strip(matrix, (tessellation * 2) + 2, innervx, innervy);

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        for i in 0..tessellation {
            let angle = r_start + (unit_angle * i as f32);

            innervx = (angle.sin() * inner_width) + position.0;
            innervy = (angle.cos() * inner_height) + position.1;

            let vx = (angle.sin() * width) + position.0;
            let vy = (angle.cos() * height) + position.1;

            rp.add_vertex(matrix, innervx, innervy, colour, uvm.map[4], uvm.map[5]);
            rp.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);
        }

        let angle: f32 = r_end;
        innervx = (angle.sin() * inner_width) + position.0;
        innervy = (angle.cos() * inner_height) + position.1;

        let vx = (angle.sin() * width) + position.0;
        let vy = (angle.cos() * height) + position.1;

        rp.add_vertex(matrix, innervx, innervy, colour, uvm.map[4], uvm.map[5]);
        rp.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);

        Ok(())
    }

    pub fn render_poly(
        &mut self,
        matrix: &Matrix,
        coords: &[Var],
        colours: &[Var],
        uvm: &UvMapping,
    ) -> Result<()> {
        let num_vertices = coords.len();
        if colours.len() != num_vertices {
            return Err(Error::Bind(
                "render_poly: coords and colours length mismatch".to_string(),
            ));
        } else if num_vertices < 3 {
            return Ok(());
        }

        if let Var::V2D(x, y) = coords[0] {
            self.prepare_to_add_triangle_strip(matrix, num_vertices, x, y);
        }

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        for i in 0..num_vertices {
            if let Var::Colour(col) = colours[i] {
                if col.format == ColourFormat::Rgb {
                    if let Var::V2D(x, y) = coords[i] {
                        rp.add_vertex(matrix, x, y, &col, uvm.map[4], uvm.map[5])
                    }
                } else {
                    let rgb = col.convert(ColourFormat::Rgb)?;
                    if let Var::V2D(x, y) = coords[i] {
                        rp.add_vertex(matrix, x, y, &rgb, uvm.map[4], uvm.map[5])
                    }
                }
            }
        }

        Ok(())
    }

    pub fn render_quadratic(
        &mut self,
        matrix: &Matrix,
        coords: &[f32; 6],
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
        let y0 = coords[1];
        let y1 = coords[3];
        let y2 = coords[5];

        let unit = (t_end - t_start) / (tessellation as f32 - 1.0);

        let tex_t = 1.0 / tessellation as f32;

        let rgb = colour.convert(ColourFormat::Rgb)?;

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
        self.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y);
        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

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
            rp.add_vertex(matrix, v1x, v1y, &rgb, u, v);
            let u = lerp(uv_t, au, cu);
            let v = lerp(uv_t, av, cv);
            rp.add_vertex(matrix, v2x, v2y, &rgb, u, v);
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

        rp.add_vertex(matrix, v1x, v1y, &rgb, du, dv);

        rp.add_vertex(matrix, v2x, v2y, &rgb, cu, cv);

        Ok(())
    }

    pub fn render_quadratic_vars(
        &mut self,
        matrix: &Matrix,
        coords: &Vec<Var>,
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

        let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 0 should be a Vec::V2D".to_string()));
        };
        let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 1 should be a Vec::V2D".to_string()));
        };
        let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 2 should be a Vec::V2D".to_string()));
        };

        let unit = (t_end - t_start) / (tessellation as f32 - 1.0);

        let tex_t = 1.0 / tessellation as f32;

        let rgb = colour.convert(ColourFormat::Rgb)?;

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
        self.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y);
        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

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
            rp.add_vertex(matrix, v1x, v1y, &rgb, u, v);
            let u = lerp(uv_t, au, cu);
            let v = lerp(uv_t, av, cv);
            rp.add_vertex(matrix, v2x, v2y, &rgb, u, v);
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

        rp.add_vertex(matrix, v1x, v1y, &rgb, du, dv);

        rp.add_vertex(matrix, v2x, v2y, &rgb, cu, cv);

        Ok(())
    }

    pub fn render_bezier(
        &mut self,
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
        self.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y);
        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

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

    pub fn render_bezier_vars(
        &mut self,
        matrix: &Matrix,
        coords: &Vec<Var>,
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

        let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 0 should be a Vec::V2D".to_string()));
        };
        let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 1 should be a Vec::V2D".to_string()));
        };
        let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 2 should be a Vec::V2D".to_string()));
        };
        let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 3 should be a Vec::V2D".to_string()));
        };


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
        self.prepare_to_add_triangle_strip(matrix, tessellation * 2, v1x, v1y);
        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

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

    pub fn render_bezier_bulging(
        &mut self,
        matrix: &Matrix,
        coords: &[f32; 8],
        line_width: f32,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        uvm: &UvMapping,
    ) -> Result<()> {
        let t_mid = (t_start + t_end) / 2.0;
        let new_tess = tessellation >> 1;

        // thin_fat
        self.render_bezier(
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
        self.render_bezier(
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

    pub fn render_bezier_bulging_vars(
        &mut self,
        matrix: &Matrix,
        coords: &Vec<Var>,
        line_width: f32,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        uvm: &UvMapping,
    ) -> Result<()> {
        let t_mid = (t_start + t_end) / 2.0;
        let new_tess = tessellation >> 1;

        // thin_fat
        self.render_bezier_vars(
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
        self.render_bezier_vars(
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

    pub fn render_stroked_bezier(
        &mut self,
        matrix: &Matrix,
        tessellation: usize,
        coords: &[f32; 8],
        stroke_tessellation: usize,
        stroke_noise: f32,
        stroke_line_width_start: f32,
        stroke_line_width_end: f32,
        colour: &Colour,
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

        let mut lab = colour.convert(ColourFormat::Lab)?;
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

            self.render_quadratic(
                matrix,
                &quad_coords,
                stroke_line_width_start,
                stroke_line_width_end,
                mapping,
                0.0,
                1.0,
                &lab,
                stroke_tessellation,
                uvm,
            )?;
        }
        Ok(())
    }

    pub fn render_stroked_bezier_vars(
        &mut self,
        matrix: &Matrix,
        tessellation: usize,
        coords: &Vec<Var>,
        stroke_tessellation: usize,
        stroke_noise: f32,
        stroke_line_width_start: f32,
        stroke_line_width_end: f32,
        colour: &Colour,
        colour_volatility: f32,
        seed: f32,
        mapping: Easing,
        uvm: &UvMapping,
    ) -> Result<()> {
        let (x1, y1) = if let Var::V2D(x, y) = coords[0] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 0 should be a Vec::V2D".to_string()));
        };
        let (x2, y2) = if let Var::V2D(x, y) = coords[1] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 1 should be a Vec::V2D".to_string()));
        };
        let (x3, y3) = if let Var::V2D(x, y) = coords[2] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 2 should be a Vec::V2D".to_string()));
        };
        let (x4, y4) = if let Var::V2D(x, y) = coords[3] {
            (x, y)
        } else {
            return Err(Error::Geometry("coords 3 should be a Vec::V2D".to_string()));
        };

        let si_num = tessellation + 2;
        let si_unit = 1.0 / (si_num as f32 - 1.0);

        let mut lab = colour.convert(ColourFormat::Lab)?;
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

            self.render_quadratic(
                matrix,
                &quad_coords,
                stroke_line_width_start,
                stroke_line_width_end,
                mapping,
                0.0,
                1.0,
                &lab,
                stroke_tessellation,
                uvm,
            )?;
        }
        Ok(())
    }

    pub fn render_stroked_bezier_rect(
        &mut self,
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
        colour: &Colour,
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

        let mut half_alpha_col = colour.convert(ColourFormat::Lab)?;
        half_alpha_col.e3 *= 0.5;

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

            self.render_stroked_bezier(
                matrix,
                tessellation,
                &coords,
                stroke_tessellation,
                stroke_noise,
                stroke_thickness,
                stroke_thickness,
                &half_alpha_col,
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

            self.render_stroked_bezier(
                matrix,
                tessellation,
                &coords,
                stroke_tessellation,
                stroke_noise,
                stroke_thickness,
                stroke_thickness,
                &half_alpha_col,
                colour_volatility,
                prng.prng_f32(),
                Easing::Linear,
                uvm,
            )?;
        }
        Ok(())
    }
}
