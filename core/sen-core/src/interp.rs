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
use crate::vm::InterpStateStruct;

pub fn interp_from_struct(t: f32, interp_state: &InterpStateStruct) -> f32 {
    let from_interp = (interp_state.from_m * t) + interp_state.from_c;
    let to_interp = easing(from_interp, interp_state.mapping);
    let res = (interp_state.to_m * to_interp) + interp_state.to_c;

    if interp_state.clamping {
        return if from_interp < 0.0 {
            interp_state.to.0
        } else if from_interp > 1.0 {
            interp_state.to.1
        } else {
            res
        };
    }

    res
}

pub fn interp_scalar(a: f32, b: f32, mapping: Easing, clamping: bool, t: f32) -> f32 {
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

pub fn interp_cos(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).cos()
}

pub fn interp_sin(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).sin()
}

pub fn interp_bezier(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        bezier_point(coords[0], coords[2], coords[4], coords[6], t),
        bezier_point(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn interp_bezier_tangent(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        bezier_tangent(coords[0], coords[2], coords[4], coords[6], t),
        bezier_tangent(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn interp_circle(position: (f32, f32), radius: f32, t: f32) -> (f32, f32) {
    let angle = t * TAU;

    (
        (angle.sin() * radius) + position.0,
        (angle.cos() * radius) + position.1,
    )
}

pub fn interp_ray(point: (f32, f32), direction: (f32, f32), t: f32) -> (f32, f32) {
    // direction should be a normalized vector
    (point.0 + (direction.0 * t), point.1 + (direction.1 * t))
}

#[cfg(test)]
mod tests {
    use crate::vm::tests::*;

    #[test]
    fn test_interp_cos() {
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos t: (/ x 5))))",
            "1 0.9800666 0.921061 0.8253356 0.6967067 0.5403023",
        );
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos amplitude: 3 t: (/ x 5))))",
            "3 2.9401999 2.7631829 2.476007 2.09012 1.6209068",
        );
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos frequency: 7 t: (/ x 5))))",
            "1 0.16996716 -0.9422223 -0.49026057 0.7755658 0.75390226",
        );
    }

    #[test]
    fn test_interp_sin() {
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin t: (/ x 5))))",
            "0 0.19866933 0.38941833 0.5646425 0.7173561 0.84147096",
        );
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin amplitude: 3 t: (/ x 5))))",
            "0 0.596008 1.168255 1.6939275 2.1520681 2.5244129",
        );
        is_debug_str(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin frequency: 7 t: (/ x 5))))",
            "0 0.98544973 0.3349882 -0.8715759 -0.6312667 0.6569866",
        );
    }

    #[test]
    fn test_interp_build() {
        is_debug_str(
            "(define i (interp/build from: [0 1] to: [0 100]))
             (probe scalar: (interp/value from: i t: 0.5))",
            "50",
        );
        is_debug_str(
            "(define i (interp/build from: [10 20] to: [50 200]))
             (probe scalar: (interp/value from: i t: 10.0))",
            "50",
        );
        is_debug_str(
            "(define i (interp/build from: [10 20] to: [50 200]))
             (probe scalar: (interp/value from: i t: 20.0))",
            "200",
        );
        is_debug_str(
            "(define i (interp/build from: [50 10] to: [100 1000]))
             (probe scalar: (interp/value from: i t: 50.0))",
            "100",
        );
        is_debug_str(
            "(define i (interp/build from: [50 10] to: [100 1000]))
             (probe scalar: (interp/value from: i t: 10.0))",
            "1000",
        );

        // clamping
        is_debug_str(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: false))
             (probe scalar: (interp/value from: i t: 2.0))",
            "200",
        );
        is_debug_str(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: true))
             (probe scalar: (interp/value from: i t: 2.0))",
            "100",
        );
        is_debug_str(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: true))
             (probe scalar: (interp/value from: i t: -2.0))",
            "0",
        );
    }

}
