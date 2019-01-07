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

use crate::uvmapper::{Mappings, BrushType};

pub struct Geometry {
    pub geo: Vec<f32>,
}

fn tex_x(v: i32) -> f32 {
    v as f32 / 1024.0
}

fn tex_y(v: i32) -> f32 {
    v as f32 / 1024.0
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            // todo: work out reasonable defaults
            geo: Vec::with_capacity(4096),
        }
    }

    pub fn test_render(&mut self) {
        let x = 10.0;
        let y = 10.0;
        let w = 980.0;
        let h = 980.0;

        let mappings = Mappings::new();
        let mapping = mappings.get_uv_mapping(BrushType::Flat, 0);
        let map = mapping.map();

        self.push(x,     y,     1.0, 0.0, 0.0, 1.0, map[0], map[1]);
        self.push(x + w, y,     1.0, 0.0, 0.0, 1.0, map[2], map[3]);
        self.push(x,     y + h, 1.0, 0.0, 0.0, 1.0, map[4], map[5]);
        self.push(x + w, y + h, 1.0, 0.0, 0.0, 1.0, map[6], map[7]);
    }

    pub fn get_render_packet_geo_len(&self, _packet_number: i32) -> usize {
        self.geo.len()
    }

    pub fn get_render_packet_geo_ptr(&self, _packet_number: i32) -> *const f32 {
        self.geo.as_ptr() as *const f32
    }

    pub fn push(&mut self, x: f32, y: f32, r: f32, g: f32, b: f32, a: f32, u: f32, v: f32) {
        self.geo.append(&mut vec![x, y, r, g, b, a, u, v]);
    }

    // duplicate the last geometry point
    fn dup(&mut self) {
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

        self.push(x, y, r, g, b, a, u, v);
    }
}
