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
    uvm: &UvMapping,
) -> Result<()> {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    render_list.prepare_to_add_triangle_strip(
        matrix,
        4,
        position.0 - half_width,
        position.1 - half_height,
    )?;

    let rp = render_list
        .render_packets
        .last_mut()
        .ok_or(Error::Geometry)?;
    let rpg = rp.get_mut_render_packet_geometry()?;

    rpg.add_vertex(
        matrix,
        position.0 - half_width,
        position.1 - half_height,
        colour,
        uvm.map[0],
        uvm.map[1],
    );
    rpg.add_vertex(
        matrix,
        position.0 + half_width,
        position.1 - half_height,
        colour,
        uvm.map[2],
        uvm.map[3],
    );
    rpg.add_vertex(
        matrix,
        position.0 - half_width,
        position.1 + half_height,
        colour,
        uvm.map[4],
        uvm.map[5],
    );
    rpg.add_vertex(
        matrix,
        position.0 + half_width,
        position.1 + half_height,
        colour,
        uvm.map[6],
        uvm.map[7],
    );

    Ok(())
}
