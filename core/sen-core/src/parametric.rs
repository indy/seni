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

use crate::ease::*;
use crate::mathutil::*;

pub fn parametric_scalar(a: f32, b: f32, mapping: Easing, clamping: bool, t: f32) -> f32 {
    let new_t = easing(t, mapping);
    let res = lerp(new_t, a, b);

    if clamping {
        if new_t < 0.0 {
            return a;
        } else if new_t > 1.0 {
            return b;
        } else {
            return res;
        }
    }

    return res;
}

pub fn parametric_cos(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).cos()
}

pub fn parametric_sin(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).sin()
}

pub fn parametric_bezier(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        bezier_point(coords[0], coords[2], coords[4], coords[6], t),
        bezier_point(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn parametric_bezier_tangent(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        bezier_tangent(coords[0], coords[2], coords[4], coords[6], t),
        bezier_tangent(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn parametric_circle(position: (f32, f32), radius: f32, t: f32) -> (f32, f32) {
    let angle = t * TAU;

    (
        (angle.sin() * radius) + position.0,
        (angle.cos() * radius) + position.1,
    )
}

pub fn parametric_ray(point: (f32, f32), direction: (f32, f32), t: f32) -> (f32, f32) {
    // direction should be a normalized vector
    (point.0 + (direction.0 * t), point.1 + (direction.1 * t))
}
