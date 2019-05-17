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

use crate::colour::Colour;
use crate::matrix::Matrix;

pub mod bezier;
pub mod bezier_bulging;
pub mod circle;
pub mod circle_slice;
pub mod line;
pub mod poly;
pub mod quadratic;
pub mod rect;
pub mod stroked_bezier;
pub mod stroked_bezier_rect;

const RENDER_PACKET_MAX_SIZE: usize = 262_144;
pub const RENDER_PACKET_FLOAT_PER_VERTEX: usize = 8;
// 262144 * 4 == 1MB per render packet
// 262144 / 8 == 32768 vertices per render packet

pub struct RenderPacket {
    pub geo: Vec<f32>,
}

impl Default for RenderPacket {
    fn default() -> RenderPacket {
        RenderPacket {
            geo: Vec::with_capacity(RENDER_PACKET_MAX_SIZE),
        }
    }
}

impl RenderPacket {
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

pub struct Geometry {
    pub render_packets: Vec<RenderPacket>,
}

impl Default for Geometry {
    fn default() -> Geometry {
        let mut render_packets: Vec<RenderPacket> = Vec::new();
        render_packets.push(Default::default());

        Geometry { render_packets }
    }
}

impl Geometry {
    pub fn reset(&mut self) {
        self.render_packets.clear();
        self.render_packets.push(Default::default())
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

    pub fn prepare_to_add_triangle_strip(
        &mut self,
        matrix: &Matrix,
        num_vertices: usize,
        x: f32,
        y: f32,
    ) {
        let mut last = self.render_packets.len() - 1;
        let mut rp = &mut self.render_packets[last];
        if !rp.can_vertices_fit(num_vertices) {
            self.render_packets.push(Default::default());
            last += 1;
        }

        rp = &mut self.render_packets[last];
        if !rp.is_empty() {
            rp.form_degenerate_triangle(matrix, x, y);
        }
    }
}
