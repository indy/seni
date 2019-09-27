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
use crate::matrix::Matrix;
use crate::rgb::Rgb;
use log::error;

pub mod bezier;
pub mod bezier_bulging;
pub mod circle;
pub mod circle_slice;
pub mod line;
pub mod poly;
pub mod quadratic;
pub mod rect;
pub mod stroked_bezier;

const RENDER_PACKET_MAX_SIZE: usize = 262_144;
pub const RENDER_PACKET_FLOAT_PER_VERTEX: usize = 8;
// 262144 * 4 == 1MB per render packet
// 262144 / 8 == 32768 vertices per render packet

pub struct RenderPacketGeometry {
    pub geo: Vec<f32>,
}

pub struct RenderPacketMask {
    pub filename: String,
    pub invert: bool,
}

// the final image
pub struct RenderPacketImage {
    pub linear_colour_space: bool,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
}

pub enum RenderPacket {
    Geometry(RenderPacketGeometry),
    Mask(RenderPacketMask),
    Image(RenderPacketImage),
}

// explicitly number the enum values to make sure they match up with values on the client-side
//
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RPCommand {
    Geometry = 1,
    Mask = 2,
    Image = 3,
}

impl RenderPacket {
    pub fn get_mut_render_packet_geometry(&mut self) -> Result<&mut RenderPacketGeometry> {
        match self {
            RenderPacket::Geometry(rpg) => Ok(rpg),
            _ => Err(Error::Geometry),
        }
    }
}

impl RenderPacketMask {
    pub fn new() -> RenderPacketMask {
        RenderPacketMask {
            filename: "".to_string(),
            invert: false,
        }
    }
}

impl RenderPacketGeometry {
    pub fn new() -> RenderPacketGeometry {
        RenderPacketGeometry {
            geo: Vec::with_capacity(RENDER_PACKET_MAX_SIZE),
        }
    }

    pub fn get_geo_len(&self) -> usize {
        self.geo.len()
    }

    pub fn get_geo_ptr(&self) -> *const f32 {
        self.geo.as_ptr() as *const f32
    }

    pub fn add_vertex(&mut self, matrix: &Matrix, x: f32, y: f32, col: &Rgb, u: f32, v: f32) {
        // assuming that col is ColourFormat::Rgb

        let (nx, ny) = matrix.transform_vec2(x, y);

        // note: the shader should pre-multiply the r,g,b elements by alpha
        self.geo
            .append(&mut vec![nx, ny, col.0, col.1, col.2, col.3, u, v]);
    }

    pub fn form_degenerate_triangle(&mut self, matrix: &Matrix, x: f32, y: f32) {
        // just copy the previous entries
        self.dup();

        // add the new vertex to complete the degenerate triangle
        let rgb = Rgb::new(0.0, 0.0, 0.0, 0.0);
        self.add_vertex(matrix, x, y, &rgb, 0.0, 0.0);

        // Note: still need to call addVertex on the first
        // vertex when we 'really' render the strip
    }

    // duplicate the last geometry point
    pub fn dup(&mut self) {
        let len = self.geo.len();

        let x: f32;
        let y: f32;
        let r: f32;
        let g: f32;
        let b: f32;
        let a: f32;
        let u: f32;
        let v: f32;
        {
            x = self.geo[len - 8];
            y = self.geo[len - 7];
            r = self.geo[len - 6];
            g = self.geo[len - 5];
            b = self.geo[len - 4];
            a = self.geo[len - 3];
            u = self.geo[len - 2];
            v = self.geo[len - 1];
        }

        self.geo.append(&mut vec![x, y, r, g, b, a, u, v]);
    }

    pub fn can_vertices_fit(&self, num: usize) -> bool {
        self.geo.len() < (RENDER_PACKET_MAX_SIZE - (num * RENDER_PACKET_FLOAT_PER_VERTEX))
    }

    pub fn is_empty(&self) -> bool {
        self.geo.is_empty()
    }
}

pub struct Geometry {
    pub render_packets: Vec<RenderPacket>,
}

impl Default for Geometry {
    fn default() -> Geometry {
        let mut render_packets: Vec<RenderPacket> = Vec::new();
        render_packets.push(RenderPacket::Geometry(RenderPacketGeometry::new()));
        Geometry { render_packets }
    }
}

impl Geometry {
    pub fn reset(&mut self) {
        self.render_packets.clear();
        self.render_packets
            .push(RenderPacket::Geometry(RenderPacketGeometry::new()))
    }

    pub fn push_rp_mask(&mut self, render_packet_mask: RenderPacketMask) -> Result<()> {
        self.render_packets
            .push(RenderPacket::Mask(render_packet_mask));

        Ok(())
    }

    pub fn push_rp_image(&mut self, render_packet_image: RenderPacketImage) -> Result<()> {
        self.render_packets
            .push(RenderPacket::Image(render_packet_image));

        Ok(())
    }

    pub fn get_render_packet_command(&self, packet_number: usize) -> Result<RPCommand> {
        let rp = &self.render_packets[packet_number];
        let res = match rp {
            RenderPacket::Geometry(_) => RPCommand::Geometry,
            RenderPacket::Mask(_) => RPCommand::Mask,
            RenderPacket::Image(_) => RPCommand::Image,
        };

        Ok(res)
    }

    pub fn get_render_packet_geometry(
        &self,
        packet_number: usize,
    ) -> Result<&RenderPacketGeometry> {
        let rp = &self.render_packets[packet_number];
        match rp {
            RenderPacket::Geometry(rpg) => Ok(rpg),
            _ => Err(Error::Geometry),
        }
    }

    pub fn get_render_packet_mask(&self, packet_number: usize) -> Result<&RenderPacketMask> {
        let rp = &self.render_packets[packet_number];
        match rp {
            RenderPacket::Mask(rpm) => Ok(rpm),
            _ => Err(Error::Geometry),
        }
    }

    pub fn get_render_packet_image(&self, packet_number: usize) -> Result<&RenderPacketImage> {
        let rp = &self.render_packets[packet_number];
        match rp {
            RenderPacket::Image(rpi) => Ok(rpi),
            _ => Err(Error::Geometry),
        }
    }

    // the one place for cleaning up the render packets before they're sent off for rendering
    // do it here rather than spreading the complexity throughout all of the different commands
    //
    pub fn remove_useless_render_packets(&mut self) {
        self.render_packets.retain(|rp| match rp {
            RenderPacket::Geometry(rpg) => !rpg.geo.is_empty(),
            RenderPacket::Mask(_) => true,
            RenderPacket::Image(_) => true,
        });

        // for (index, rp) in self.render_packets.iter().enumerate() {
        //     if rp.command == RPCommand::RenderGeometry {
        //         error!(
        //             "cleanedup render packet {} RenderGeometry len: {}",
        //             index,
        //             rp.geo.len()
        //         );
        //     } else if rp.command == RPCommand::Mask {
        //         error!(
        //             "cleanedup render packet {} Mask {}",
        //             index, rp.mask_filename
        //         );
        //     }
        // }
    }

    pub fn get_num_render_packets(&self) -> usize {
        self.render_packets.len()
    }

    pub fn prepare_to_add_triangle_strip(
        &mut self,
        matrix: &Matrix,
        num_vertices: usize,
        x: f32,
        y: f32,
    ) -> Result<()> {
        let rp = self.render_packets.last_mut().ok_or(Error::Geometry)?;

        match rp {
            RenderPacket::Geometry(rpg) => {
                if !rpg.can_vertices_fit(num_vertices) {
                    if num_vertices >= RENDER_PACKET_MAX_SIZE {
                        error!(
                            "prepare_to_add_triangle_strip trying to add more than {} vertices",
                            RENDER_PACKET_MAX_SIZE
                        );
                        return Err(Error::Geometry);
                    }

                    self.render_packets
                        .push(RenderPacket::Geometry(RenderPacketGeometry::new()));
                    return Ok(());
                }

                if !rpg.is_empty() {
                    rpg.form_degenerate_triangle(matrix, x, y);
                }
                Ok(())
            }
            _ => {
                self.render_packets
                    .push(RenderPacket::Geometry(RenderPacketGeometry::new()));
                return Ok(());
            }
        }
    }
}
