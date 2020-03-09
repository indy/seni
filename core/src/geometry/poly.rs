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
use log::error;

pub fn render(
    render_list: &mut RenderList,
    matrix: &Matrix,
    coords: &[(f32, f32)],
    colours: &[Rgb],
    uvm: &UvMapping,
) -> Result<()> {
    let num_vertices = coords.len();
    if colours.len() != num_vertices {
        error!("render_poly: coords and colours length mismatch");
        return Err(Error::Geometry);
    } else if num_vertices < 3 {
        return Ok(());
    }

    let (x, y) = coords[0];
    render_list.prepare_to_add_triangle_strip(matrix, num_vertices, x, y)?;

    let rp = render_list
        .render_packets
        .last_mut()
        .ok_or(Error::Geometry)?;
    let rpg = rp.get_mut_render_packet_geometry()?;

    for i in 0..num_vertices {
        let (x, y) = coords[i];
        rpg.add_vertex(matrix, x, y, &colours[i], uvm.map[4], uvm.map[5])
    }

    Ok(())
}
