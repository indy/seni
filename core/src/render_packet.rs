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

use crate::error::{Error, Result};
use crate::matrix::Matrix;
use crate::rgb::Rgb;

pub const RENDER_PACKET_MAX_SIZE: usize = 262_144;
pub const RENDER_PACKET_FLOAT_PER_VERTEX: usize = 8;
// 262144 * 4 == 1MB per render packet
// 262144 / 8 == 32768 vertices per render packet

#[derive(Default)]
pub struct RenderPacketGeometry {
    pub geo: Vec<f32>,
}

#[derive(Default)]
pub struct RenderPacketMask {
    pub filename: String,
    pub invert: bool,
}

// the final image
pub struct RenderPacketImage {
    pub linear_colour_space: bool,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
}

pub enum RenderPacket {
    Geometry(RenderPacketGeometry),
    Mask(RenderPacketMask),
    Image(RenderPacketImage),
}

impl RenderPacket {
    pub fn get_mut_render_packet_geometry(&mut self) -> Result<&mut RenderPacketGeometry> {
        match self {
            RenderPacket::Geometry(rpg) => Ok(rpg),
            _ => Err(Error::Geometry),
        }
    }
}

impl RenderPacketMask {
    pub fn new() -> RenderPacketMask {
        RenderPacketMask {
            filename: "".to_string(),
            invert: false,
        }
    }
}

impl RenderPacketGeometry {
    pub fn new() -> RenderPacketGeometry {
        RenderPacketGeometry {
            geo: Vec::with_capacity(RENDER_PACKET_MAX_SIZE),
        }
    }

    pub fn get_geo_len(&self) -> usize {
        self.geo.len()
    }

    pub fn get_geo_ptr(&self) -> *const f32 {
        self.geo.as_ptr() as *const f32
    }

    pub fn add_vertex(&mut self, matrix: &Matrix, x: f32, y: f32, col: &Rgb, u: f32, v: f32) {
        // assuming that col is ColourFormat::Rgb

        let (nx, ny) = matrix.transform_vec2(x, y);

        // note: the shader should pre-multiply the r,g,b elements by alpha
        self.geo
            .append(&mut vec![nx, ny, col.0, col.1, col.2, col.3, u, v]);
    }

    pub fn form_degenerate_triangle(&mut self, matrix: &Matrix, x: f32, y: f32) {
        // just copy the previous entries
        self.dup();

        // add the new vertex to complete the degenerate triangle
        let rgb = Rgb::new(0.0, 0.0, 0.0, 0.0);
        self.add_vertex(matrix, x, y, &rgb, 0.0, 0.0);

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
