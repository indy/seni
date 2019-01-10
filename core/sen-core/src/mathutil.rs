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

pub const PI: f32 = std::f32::consts::PI;
pub const PI_BY_2: f32 = std::f32::consts::FRAC_PI_2;
pub const TAU: f32 = std::f32::consts::PI * 2.0;

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
    } else if x > xmax {
        xmax
    } else {
        x
    }
}

pub fn map_quick_ease(x: f32) -> f32 {
  let x2 = x * x;
  let x3 = x * x * x;

  (3.0 * x2) - (2.0 * x3)
}

pub fn map_slow_ease_in(x: f32) -> f32 {
  let s = (x * PI_BY_2).sin();
  s * s * s * s
}

pub fn map_slow_ease_in_ease_out(x: f32) -> f32 {
    x - ((x * TAU).sin() / TAU)
}

pub fn length_v2(x: f32, y: f32) -> f32 {
    ((x * x) + (y * y)).sqrt()
}

pub fn distance_v2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let xdiff = ax - bx;
    let ydiff = ay - by;

    length_v2(xdiff, ydiff)
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

pub fn opposite_normal(x: f32, y: f32) -> (f32, f32) {
    (-x, -y)
}

pub fn quadratic_point(a: f32, b: f32, c: f32, t: f32) -> f32 {
    let r = ((b - a) - 0.5 * (c - a)) / (0.5 * (0.5 - 1.0));
    let s = c - a - r;

    (r * t * t) + (s * t) + a
}

pub fn bezier_point(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let t1 = 1.0 - t;
    (a * t1 * t1 * t1) + (3.0 * b * t * t1 * t1) + (3.0 * c * t * t * t1) + (d * t * t * t)
}

pub fn bezier_tangent(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    3.0 * t * t * (-a + 3.0 * b - 3.0 * c + d) + 6.0 * t * (a - 2.0 * b + c) + 3.0 * (-a + b)
}
