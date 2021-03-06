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

use crate::ease;
use crate::mathutil;

#[derive(Clone, Debug)]
pub struct InterpStateStruct {
    pub from_a: f32,
    pub from_b: f32,
    pub to_a: f32,
    pub to_b: f32,
    pub clamping: bool,
    pub mapping: ease::Easing,
}

impl Default for InterpStateStruct {
    fn default() -> InterpStateStruct {
        InterpStateStruct {
            from_a: 0.0,
            from_b: 0.0,
            to_a: 0.0,
            to_b: 0.0,
            clamping: false,
            mapping: ease::Easing::Linear,
        }
    }
}

impl InterpStateStruct {
    pub fn value(&self, t: f32) -> f32 {
        parametric(t, self.from_a, self.from_b, self.to_a, self.to_b, self.mapping, self.clamping)
    }
}

pub fn parametric(
    val: f32,
    from_a: f32,
    from_b: f32,
    to_a: f32,
    to_b: f32,
    mapping: ease::Easing,
    clamping: bool,
) -> f32 {
    let from_t = mathutil::unlerp(val, from_a, from_b);
    let to_t = ease::easing(from_t, mapping);
    let res = mathutil::lerp(to_t, to_a, to_b);

    if clamping {
        return if to_t < 0.0 {
            to_a
        } else if to_t > 1.0 {
            to_b
        } else {
            res
        };
    }

    res
}

pub fn scalar(a: f32, b: f32, mapping: ease::Easing, clamping: bool, t: f32) -> f32 {
    let new_t = ease::easing(t, mapping);
    let res = mathutil::lerp(new_t, a, b);

    if clamping {
        if new_t < 0.0 {
            return a;
        } else if new_t > 1.0 {
            return b;
        } else {
            return res;
        }
    }

    res
}

pub fn cos(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).cos()
}

pub fn sin(amplitude: f32, frequency: f32, t: f32) -> f32 {
    amplitude * (t * frequency).sin()
}

pub fn bezier(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        mathutil::bezier_point(coords[0], coords[2], coords[4], coords[6], t),
        mathutil::bezier_point(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn bezier_tangent(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        mathutil::bezier_tangent(coords[0], coords[2], coords[4], coords[6], t),
        mathutil::bezier_tangent(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn circle(position: (f32, f32), radius: f32, t: f32) -> (f32, f32) {
    let angle = t * mathutil::TAU;

    (
        (angle.sin() * radius) + position.0,
        (angle.cos() * radius) + position.1,
    )
}

pub fn ray(point: (f32, f32), direction: (f32, f32), t: f32) -> (f32, f32) {
    // direction should be a normalized vector
    (point.0 + (direction.0 * t), point.1 + (direction.1 * t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::tests::*;

    #[test]
    fn test_interp_parametric() {
        assert_eq!(parametric(20.0, 5.0, 50.0, 10.0, 100.0, ease::Easing::Linear, false), 40.0);
        assert_eq!(parametric(10.0, 5.0, 50.0, 20.0, 100.0, ease::Easing::SlowIn, false), 20.07274);
        assert_eq!(parametric(45.0, 5.0, 50.0, 20.0, 100.0, ease::Easing::CubicOut, false), 99.89026);
        assert_eq!(parametric(2.0, 5.0, 50.0, 10.0, 100.0, ease::Easing::Linear, true), 10.0);
        assert_eq!(parametric(20.0, 10.0, 100.0, 5.0, 50.0, ease::Easing::Linear, false), 10.0);
        assert_eq!(parametric(10.0, 20.0, 100.0, 5.0, 50.0, ease::Easing::SlowIn, false), 5.065186);
        assert_eq!(parametric(45.0, 20.0, 100.0, 5.0, 50.0, ease::Easing::CubicOut, false), 35.377197);
        assert_eq!(parametric(2.0, 10.0, 100.0, 5.0, 50.0, ease::Easing::Linear, true), 5.0);
    }

    #[test]
    fn test_interp_cos() {
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos t: (/ x 5))))",
            [1.0, 0.9800666, 0.921061, 0.8253356, 0.6967067, 0.5403023].to_vec(),
        );

        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos t: (/ x 5))))",
            [1.0, 0.9800666, 0.921061, 0.8253356, 0.6967067, 0.5403023].to_vec(),
        );
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos amplitude: 3 t: (/ x 5))))",
            [3.0, 2.9401999, 2.7631829, 2.476007, 2.09012, 1.6209068].to_vec(),
        );
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/cos frequency: 7 t: (/ x 5))))",
            [
                1.0,
                0.16996716,
                -0.9422223,
                -0.49026057,
                0.7755658,
                0.75390226,
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_interp_sin() {
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin t: (/ x 5))))",
            [
                0.0, 0.19866933, 0.38941833, 0.5646425, 0.7173561, 0.84147096,
            ]
            .to_vec(),
        );
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin amplitude: 3 t: (/ x 5))))",
            [0.0, 0.596008, 1.168255, 1.6939275, 2.1520681, 2.5244129].to_vec(),
        );
        probe_has_scalars(
            "(loop (x upto: 5)
                   (probe scalar: (interp/sin frequency: 7 t: (/ x 5))))",
            [
                0.0, 0.98544973, 0.3349882, -0.8715759, -0.6312667, 0.6569866,
            ]
            .to_vec(),
        );
    }

    #[test]
    fn test_interp_build() {
        probe_has_scalars(
            "(define i (interp/build from: [0 1] to: [0 100]))
             (probe scalar: (interp/value from: i t: 0.5))",
            [50.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [10 20] to: [50 200]))
             (probe scalar: (interp/value from: i t: 10.0))",
            [50.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [10 20] to: [50 200]))
             (probe scalar: (interp/value from: i t: 20.0))",
            [200.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [50 10] to: [100 1000]))
             (probe scalar: (interp/value from: i t: 50.0))",
            [100.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [50 10] to: [100 1000]))
             (probe scalar: (interp/value from: i t: 10.0))",
            [1000.0].to_vec(),
        );

        // clamping
        probe_has_scalars(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: false))
             (probe scalar: (interp/value from: i t: 2.0))",
            [200.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: true))
             (probe scalar: (interp/value from: i t: 2.0))",
            [100.0].to_vec(),
        );
        probe_has_scalars(
            "(define i (interp/build from: [0 1] to: [0 100] clamping: true))
             (probe scalar: (interp/value from: i t: -2.0))",
            [0.0].to_vec(),
        );
    }
}
