// Copyright (C) 2019 Inderjit Gill

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

use crate::error::{Error, Result};
use crate::geometry::Geometry;
use crate::matrix::Matrix;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    position: (f32, f32),
    width: f32,
    height: f32,
    colour: &Rgb,
    uvm: &UvMapping,
) -> Result<()> {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    geometry.prepare_to_add_triangle_strip(
        matrix,
        4,
        position.0 - half_width,
        position.1 - half_height,
    )?;

    let rp = geometry.render_packets.last_mut().ok_or(Error::Geometry)?;
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
