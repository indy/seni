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

pub struct Geometry {
    pub geo: Vec<f32>,
    pub crt: Vec<f32>,
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            // todo: work out reasonable defaults
            geo: Vec::with_capacity(4096),
            crt: Vec::with_capacity(64),
        }
    }

    pub fn geo_len(&self) -> usize {
        self.geo.len()
    }

    pub fn geo_ptr(&self) -> *const f32 {
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
