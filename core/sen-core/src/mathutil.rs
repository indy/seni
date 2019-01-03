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

pub const PI: f32 = 3.141_592_653_589_793;
pub const PI_BY_2: f32 = 1.570_796_326_794_896;
pub const TAU: f32 = 6.283_185_307_179_586;

pub fn deg_to_rad(a: f32) -> f32 {
    a * (PI / 180.0)
}

pub fn rad_to_deg(a: f32) -> f32 {
    a * (180.0 / PI)
}

pub fn absf(x: f32) -> f32 {
    x.abs()
}

pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

pub fn unlerp(t: f32, a: f32, b: f32) -> f32 {
    (t - a) / (b - a)
}

pub fn clamp(x: f32, xmin: f32, xmax: f32) -> f32 {
    if x < xmin {
        xmin
    } else {
        if x > xmax {
            xmax
        } else {
            x
        }
    }
}

pub fn length_v2(x: f32, y: f32) -> f32 {
    ((x * x) + (y * y)).sqrt()
}

pub fn distance_v2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let xdiff = ax - bx;
    let ydiff = ay - by;

    let dist = length_v2(xdiff, ydiff);

    dist
}

pub fn normalize(x: f32, y: f32) -> (f32, f32) {
    let len = length_v2(x, y);
    (x / len, y / len)
}

pub fn normal(x1: f32, y1: f32, x2: f32, y2: f32) -> (f32, f32) {
    let dx = x2 - x1;
    let dy = y2 - y1;

    normalize(-dy, dx)
}
