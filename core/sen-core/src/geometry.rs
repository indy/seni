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

use crate::error::*;
use crate::uvmapper::{BrushType, UvMapping, Mappings};
use crate::matrix::{Matrix};
use crate::mathutil::*;
use crate::colour::Colour;

// todo: work out reasonable defaults
const RENDER_PACKET_MAX_SIZE: usize = 4096;
const RENDER_PACKET_FLOAT_PER_VERTEX: usize = 8;

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

    pub fn add_vertex(&mut self, matrix: &Matrix, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32) {
        let (nx, ny) = matrix.transform_vec2(x, y);
        // pre-multiply the alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        self.geo.append(&mut vec![nx, ny, r * a, g * a, b * a, a, u, v]);
    }

    pub fn form_degenerate_triangle(&mut self, matrix: &Matrix, x: f32, y: f32) {
        // just copy the previous entries
        self.dup();

        // add the new vertex to complete the degenerate triangle
        self.add_vertex(matrix, x, y, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

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

        Geometry {
            render_packets,
        }
    }

    pub fn test_render(&mut self, mappings: &Mappings) -> Result<(usize)> {
        let uvm = mappings.get_uv_mapping(BrushType::Flat, 0);
        self.render_line(&Matrix::identity(), 100.0, 100.0, 600.0, 600.0, 50.0,
                         &Colour::RGB(1.0, 0.0, 0.0, 1.0), &Colour::RGB(1.0, 1.0, 0.0, 1.0),
                         &uvm)?;
        let uvm2 = mappings.get_uv_mapping(BrushType::A, 0);
        self.render_line(&Matrix::identity(), 800.0, 700.0, 200.0, 100.0, 10.0,
                         &Colour::RGB(1.0, 0.0, 0.0, 1.0), &Colour::RGB(1.0, 0.0, 1.0, 1.0),
                         &uvm2)?;

        let uvm3 = mappings.get_uv_mapping(BrushType::B, 0);
        self.render_line(&Matrix::identity(), 900.0, 100.0, 900.0, 900.0, 20.0,
                         &Colour::RGB(1.0, 1.0, 0.0, 1.0), &Colour::RGB(1.0, 0.0, 1.0, 1.0),
                         &uvm3)?;

        Ok(self.render_packets.len())
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        let rp = &self.render_packets[packet_number];
        rp.geo.len()
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        let rp = &self.render_packets[packet_number];
        rp.geo.as_ptr() as *const f32
    }

    fn prepare_to_add_triangle_strip(&mut self, matrix: &Matrix, num_vertices: usize, x: f32, y: f32) {
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

    pub fn render_line(&mut self, matrix: &Matrix, from_x: f32, from_y: f32, to_x: f32, to_y: f32, width: f32, from_col: &Colour, to_col: &Colour, uvm: &UvMapping) -> Result<()> {

        let (fr, fg, fb, fa) = from_col.to_rgba32_tuple()?;
        let (tr, tg, tb, ta) = to_col.to_rgba32_tuple()?;

        let hw = (width * uvm.width_scale) / 2.0;

        let (nx, ny) = normal(from_x, from_y, to_x, to_y);
        let (n2x, n2y) = opposite_normal(nx, ny);

        self.prepare_to_add_triangle_strip(matrix, 4, from_x + (hw * nx), from_y + (hw * ny));

        let last = self.render_packets.len() - 1;
        let rp = &mut self.render_packets[last];

        rp.add_vertex(matrix, from_x + (hw * nx), from_y + (hw * ny), fr, fg, fb, fa, uvm.map[0], uvm.map[1]);
        rp.add_vertex(matrix, from_x + (hw * n2x), from_y + (hw * n2y), fr, fg, fb, fa, uvm.map[2], uvm.map[3]);
        rp.add_vertex(matrix, to_x + (hw * nx), to_y + (hw * ny), tr, tg, tb, ta, uvm.map[4], uvm.map[5]);
        rp.add_vertex(matrix, to_x + (hw * n2x), to_y + (hw * n2y), tr, tg, tb, ta, uvm.map[6], uvm.map[7]);

        Ok(())
    }

}
