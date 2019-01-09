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
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::uvmapper::UvMapping;

// todo: work out reasonable defaults
const RENDER_PACKET_MAX_SIZE: usize = 4096;
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

    pub fn add_vertex(
        &mut self,
        matrix: &Matrix,
        x: f32,
        y: f32,
        col: (f32, f32, f32, f32),
        u: f32,
        v: f32,
    ) {
        let (nx, ny) = matrix.transform_vec2(x, y);
        // pre-multiply the alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        self.geo.append(&mut vec![
            nx,
            ny,
            col.0 * col.3,
            col.1 * col.3,
            col.2 * col.3,
            col.3,
            u,
            v,
        ]);
    }

    pub fn form_degenerate_triangle(&mut self, matrix: &Matrix, x: f32, y: f32) {
        // just copy the previous entries
        self.dup();

        // add the new vertex to complete the degenerate triangle
        self.add_vertex(matrix, x, y, (0.0, 0.0, 0.0, 0.0), 0.0, 0.0);

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
        from_col: (f32, f32, f32, f32),
        to_col: (f32, f32, f32, f32),
        uvm: &UvMapping,
    ) -> Result<()> {
        // let (fr, fg, fb, fa) = from_col.to_rgba32_tuple()?;
        // let (tr, tg, tb, ta) = to_col.to_rgba32_tuple()?;

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
        colour: (f32, f32, f32, f32),
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
        colour: (f32, f32, f32, f32),
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
        colour: (f32, f32, f32, f32),
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
}
