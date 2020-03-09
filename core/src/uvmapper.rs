// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

const TEXTURE_DIM: f32 = 1024.0;

#[derive(Copy, Clone)]
pub enum BrushType {
    Flat = 0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

pub struct UvMapping {
    pub width_scale: f32,
    pub map: [f32; 8], // array of 8 (4 pairs of xy)
}

fn brush_index(brush_type: BrushType) -> usize {
    match brush_type {
        BrushType::Flat => 0,
        BrushType::A => 1,
        BrushType::B => 2,
        BrushType::C => 3,
        BrushType::D => 4,
        BrushType::E => 5,
        BrushType::F => 6,
        BrushType::G => 7,
    }
}

fn make_uv(in_u: i32, in_v: i32) -> (f32, f32) {
    (in_u as f32 / TEXTURE_DIM, in_v as f32 / TEXTURE_DIM)
}

impl UvMapping {
    fn new(width_scale: f32, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        let (m0, m1) = make_uv(max_x, min_y);
        let (m2, m3) = make_uv(max_x, max_y);
        let (m4, m5) = make_uv(min_x, min_y);
        let (m6, m7) = make_uv(min_x, max_y);

        UvMapping {
            width_scale,
            map: [m0, m1, m2, m3, m4, m5, m6, m7],
        }
    }

    pub fn map(&self) -> &[f32; 8] {
        &self.map
    }
}

pub struct Mappings {
    m: Vec<Vec<UvMapping>>,
}

impl Default for Mappings {
    fn default() -> Mappings {
        // flat
        let flat = vec![UvMapping::new(1.0, 1, 1, 2, 2)];
        let a = vec![UvMapping::new(1.2, 0, 781, 976, 1023)];
        let b = vec![
            UvMapping::new(1.4, 11, 644, 490, 782),
            UvMapping::new(1.1, 521, 621, 1023, 783),
            UvMapping::new(1.2, 340, 419, 666, 508),
            UvMapping::new(1.2, 326, 519, 659, 608),
            UvMapping::new(1.1, 680, 419, 1020, 507),
            UvMapping::new(1.1, 677, 519, 1003, 607),
        ];
        let c = vec![
            UvMapping::new(1.2, 0, 7, 324, 43),
            UvMapping::new(1.3, 0, 45, 319, 114),
            UvMapping::new(1.1, 0, 118, 328, 180),
            UvMapping::new(1.2, 0, 186, 319, 267),
            UvMapping::new(1.4, 0, 271, 315, 334),
            UvMapping::new(1.1, 0, 339, 330, 394),
            UvMapping::new(1.2, 0, 398, 331, 473),
            UvMapping::new(1.1, 0, 478, 321, 548),
            UvMapping::new(1.1, 0, 556, 326, 618),
        ];
        let d = vec![UvMapping::new(1.3, 333, 165, 734, 336)];
        let e = vec![UvMapping::new(1.3, 737, 183, 1018, 397)];
        let f = vec![UvMapping::new(1.1, 717, 2, 1023, 163)];
        let g = vec![
            UvMapping::new(1.2, 329, 0, 652, 64),
            UvMapping::new(1.0, 345, 75, 686, 140),
        ];

        Mappings {
            m: vec![flat, a, b, c, d, e, f, g],
        }
    }
}

impl Mappings {
    pub fn get_uv_mapping(&self, brush_type: BrushType, sub_type: usize) -> &UvMapping {
        // always wrap sub_type
        let index = brush_index(brush_type);

        let brush_map = &self.m[index];
        let sub = sub_type % brush_map.len();

        &brush_map[sub]
    }
}
