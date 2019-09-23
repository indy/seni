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
use crate::mathutil::*;
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
    tessellation: usize,
    angle_start: f32,
    angle_end: f32,
    inner_width: f32,
    inner_height: f32,
    uvm: &UvMapping,
) -> Result<()> {
    let r_start = deg_to_rad(angle_start);
    let r_end = deg_to_rad(angle_end);
    let unit_angle = (r_end - r_start) / tessellation as f32;

    let mut innervx = (r_start.sin() * inner_width) + position.0;
    let mut innervy = (r_start.cos() * inner_height) + position.1;

    geometry.prepare_to_add_triangle_strip(matrix, (tessellation * 2) + 2, innervx, innervy)?;

    let rp = geometry.render_packets.last_mut().ok_or(Error::Geometry)?;
    let rpg = rp.get_mut_render_packet_geometry()?;

    for i in 0..tessellation {
        let angle = r_start + (unit_angle * i as f32);

        innervx = (angle.sin() * inner_width) + position.0;
        innervy = (angle.cos() * inner_height) + position.1;

        let vx = (angle.sin() * width) + position.0;
        let vy = (angle.cos() * height) + position.1;

        rpg.add_vertex(matrix, innervx, innervy, colour, uvm.map[4], uvm.map[5]);
        rpg.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);
    }

    let angle: f32 = r_end;
    innervx = (angle.sin() * inner_width) + position.0;
    innervy = (angle.cos() * inner_height) + position.1;

    let vx = (angle.sin() * width) + position.0;
    let vy = (angle.cos() * height) + position.1;

    rpg.add_vertex(matrix, innervx, innervy, colour, uvm.map[4], uvm.map[5]);
    rpg.add_vertex(matrix, vx, vy, colour, uvm.map[4], uvm.map[5]);

    Ok(())
}
