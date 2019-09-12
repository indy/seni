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

use crate::error::Result;
use crate::geometry::Geometry;
use crate::mathutil::*;
use crate::matrix::Matrix;
use crate::rgb::Rgb;
use crate::uvmapper::UvMapping;

pub fn render(
    geometry: &mut Geometry,
    matrix: &Matrix,
    from: (f32, f32),
    to: (f32, f32),
    width: f32,
    from_col: &Rgb,
    to_col: &Rgb,
    uvm: &UvMapping,
) -> Result<()> {
    let hw = (width * uvm.width_scale) / 2.0;

    let (nx, ny) = normal(from.0, from.1, to.0, to.1);
    let (n2x, n2y) = opposite_normal(nx, ny);

    geometry.prepare_to_add_triangle_strip(matrix, 4, from.0 + (hw * nx), from.1 + (hw * ny));

    let last = geometry.render_packets.len() - 1;
    let rp = &mut geometry.render_packets[last];

    rp.add_vertex(
        matrix,
        from.0 + (hw * nx),
        from.1 + (hw * ny),
        from_col,
        uvm.map[0],
        uvm.map[1],
    );
    rp.add_vertex(
        matrix,
        from.0 + (hw * n2x),
        from.1 + (hw * n2y),
        from_col,
        uvm.map[2],
        uvm.map[3],
    );
    rp.add_vertex(
        matrix,
        to.0 + (hw * nx),
        to.1 + (hw * ny),
        to_col,
        uvm.map[4],
        uvm.map[5],
    );
    rp.add_vertex(
        matrix,
        to.0 + (hw * n2x),
        to.1 + (hw * n2y),
        to_col,
        uvm.map[6],
        uvm.map[7],
    );

    Ok(())
}
