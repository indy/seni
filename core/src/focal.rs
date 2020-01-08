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

use crate::context::Context;
use crate::ease::Easing;
use crate::interp;
use crate::mathutil;

#[derive(Copy, Clone, Debug)]
pub enum FocalType {
    Point,
    HLine,
    VLine,
}

#[derive(Clone, Debug)]
pub struct FocalStateStruct {
    pub focal_type: FocalType,
    pub mapping: Easing,
    pub position: (f32, f32),
    pub distance: f32,
    pub transform_pos: bool,
}

impl Default for FocalStateStruct {
    fn default() -> FocalStateStruct {
        FocalStateStruct {
            focal_type: FocalType::Point,
            mapping: Easing::Linear,
            position: (0.0, 0.0),
            distance: 1.0,
            transform_pos: false,
        }
    }
}

impl FocalStateStruct {
    pub fn value(&self, context: &Context, position: (f32, f32)) -> f32 {
        // transform position to canvas space coordinates
        let (x, y) = if self.transform_pos {
            context.matrix_stack.transform_vec2(position.0, position.1)
        } else {
            position
        };

        match self.focal_type {
            FocalType::Point => point(x, y, self.distance, self.mapping, self.position),
            FocalType::HLine => hline(y, self.distance, self.mapping, self.position.1),
            FocalType::VLine => vline(x, self.distance, self.mapping, self.position.0),
        }
    }
}

fn point(x: f32, y: f32, distance: f32, mapping: Easing, centre: (f32, f32)) -> f32 {
    let d = mathutil::distance_v2(x, y, centre.0, centre.1);

    if d < std::f32::EPSILON {
        return 1.0;
    }

    interp::parametric(d, 0.0, distance, 1.0, 0.0, mapping, true)
}

fn hline(y: f32, distance: f32, mapping: Easing, centre_y: f32) -> f32 {
    let d = (centre_y - y).abs();

    if d < std::f32::EPSILON {
        return 1.0;
    }

    interp::parametric(d, 0.0, distance, 1.0, 0.0, mapping, true)
}

fn vline(x: f32, distance: f32, mapping: Easing, centre_x: f32) -> f32 {
    let d = (centre_x - x).abs();

    if d < std::f32::EPSILON {
        return 1.0;
    }

    interp::parametric(d, 0.0, distance, 1.0, 0.0, mapping, true)
}
