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

use crate::colour::ColourFormat;
use crate::error::Error;
use crate::geometry::Geometry;
use crate::matrix::Matrix;
use crate::result::Result;
use crate::uvmapper::UvMapping;
use crate::vm::Var;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    coords: &[Var],
    colours: &[Var],
    uvm: &UvMapping,
) -> Result<()> {
    let num_vertices = coords.len();
    if colours.len() != num_vertices {
        return Err(Error::Bind(
            "render_poly: coords and colours length mismatch".to_string(),
        ));
    } else if num_vertices < 3 {
        return Ok(());
    }

    if let Var::V2D(x, y) = coords[0] {
        geometry.prepare_to_add_triangle_strip(matrix, num_vertices, x, y);
    }

    let last = geometry.render_packets.len() - 1;
    let rp = &mut geometry.render_packets[last];

    for i in 0..num_vertices {
        if let Var::Colour(col) = colours[i] {
            if col.format == ColourFormat::Rgb {
                if let Var::V2D(x, y) = coords[i] {
                    rp.add_vertex(matrix, x, y, &col, uvm.map[4], uvm.map[5])
                }
            } else {
                let rgb = col.convert(ColourFormat::Rgb)?;
                if let Var::V2D(x, y) = coords[i] {
                    rp.add_vertex(matrix, x, y, &rgb, uvm.map[4], uvm.map[5])
                }
            }
        }
    }

    Ok(())
}
