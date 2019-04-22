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

use crate::ease;
use crate::error::*;
use crate::mathutil;
use crate::vm::Var;

#[derive(Clone, Debug)]
pub struct InterpStateStruct {
    pub from_m: f32,
    pub to_m: f32,
    pub from_c: f32,
    pub to_c: f32,
    pub to: (f32, f32),
    pub clamping: bool,
    pub mapping: ease::Easing,
}

impl Default for InterpStateStruct {
    fn default() -> InterpStateStruct {
        InterpStateStruct {
            from_m: 0.0,
            to_m: 0.0,
            from_c: 0.0,
            to_c: 0.0,
            to: (0.0, 1.0),
            clamping: false,
            mapping: ease::Easing::Linear,
        }
    }
}

impl InterpStateStruct {
    pub fn value(&self, t: f32) -> f32 {
        let from_interp = (self.from_m * t) + self.from_c;
        let to_interp = ease::easing(from_interp, self.mapping);
        let res = (self.to_m * to_interp) + self.to_c;

        if self.clamping {
            if from_interp < 0.0 {
                self.to.0
            } else if from_interp > 1.0 {
                self.to.1
            } else {
                res
            }
        } else {
            res
        }
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
    let from_m = mathutil::mc_m(from_a, 0.0, from_b, 1.0);
    let from_c = mathutil::mc_c(from_a, 0.0, from_m);

    let to_m = mathutil::mc_m(0.0, to_a, 1.0, to_b);
    let to_c = mathutil::mc_c(0.0, to_a, to_m);

    let from_interp = (from_m * val) + from_c;
    let to_interp = ease::easing(from_interp, mapping);
    let res = (to_m * to_interp) + to_c;

    if clamping {
        return if from_interp < 0.0 {
            to_a
        } else if from_interp > 1.0 {
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

pub fn bezier_vars(coords: &Vec<Var>, t: f32) -> Result<(f32, f32)> {
    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 0 should be a Vec::V2D".to_string()));
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 1 should be a Vec::V2D".to_string()));
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 2 should be a Vec::V2D".to_string()));
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 2 should be a Vec::V2D".to_string()));
    };

    Ok((
        mathutil::bezier_point(x0, x1, x2, x3, t),
        mathutil::bezier_point(y0, y1, y2, y3, t),
    ))
}

pub fn bezier_tangent(coords: &[f32; 8], t: f32) -> (f32, f32) {
    (
        mathutil::bezier_tangent(coords[0], coords[2], coords[4], coords[6], t),
        mathutil::bezier_tangent(coords[1], coords[3], coords[5], coords[7], t),
    )
}

pub fn bezier_tangent_vars(coords: &Vec<Var>, t: f32) -> Result<(f32, f32)> {
    let (x0, y0) = if let Var::V2D(x, y) = coords[0] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 0 should be a Vec::V2D".to_string()));
    };
    let (x1, y1) = if let Var::V2D(x, y) = coords[1] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 1 should be a Vec::V2D".to_string()));
    };
    let (x2, y2) = if let Var::V2D(x, y) = coords[2] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 2 should be a Vec::V2D".to_string()));
    };
    let (x3, y3) = if let Var::V2D(x, y) = coords[3] {
        (x, y)
    } else {
        return Err(Error::Interp("coords 2 should be a Vec::V2D".to_string()));
    };

    Ok((
        mathutil::bezier_tangent(x0, x1, x2, x3, t),
        mathutil::bezier_tangent(y0, y1, y2, y3, t),
    ))
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
