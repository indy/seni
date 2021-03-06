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

use crate::error::{Error, Result};
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::render_list::RenderList;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    render_list: &mut RenderList,
    matrix: &Matrix,
    position: (f32, f32),
    width: f32,
    height: f32,
    colour: &Rgb,
    tessellation: usize,
    uvm: &UvMapping,
) -> Result<()> {
    render_list.prepare_to_add_triangle_strip(
        matrix,
        (tessellation * 2) + 2,
        position.0,
        position.1,
    )?;

    let unit_angle = TAU / tessellation as f32;

    let rp = render_list
        .render_packets
        .last_mut()
        .ok_or(Error::Geometry)?;
    let rpg = rp.get_mut_render_packet_geometry()?;

    for i in 0..tessellation {
        let angle = unit_angle * i as f32;
        let vx = (angle.sin() * width) + position.0;
        let vy = (angle.cos() * height) + position.1;

        rpg.add_vertex(
            matrix, position.0, position.1, colour, uvm.map[4], uvm.map[5],
        );
        rpg.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);
    }

    let angle: f32 = 0.0;
    let vx = (angle.sin() * width) + position.0;
    let vy = (angle.cos() * height) + position.1;

    rpg.add_vertex(
        matrix, position.0, position.1, colour, uvm.map[4], uvm.map[5],
    );
    rpg.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);

    Ok(())
}
