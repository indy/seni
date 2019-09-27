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
use crate::render_packet::{
    RPCommand, RenderPacket, RenderPacketGeometry, RenderPacketImage, RenderPacketMask,
    RENDER_PACKET_MAX_SIZE,
};
use log::error;


pub struct RenderList {
    pub render_packets: Vec<RenderPacket>,
}

impl Default for RenderList {
    fn default() -> RenderList {
        let mut render_packets: Vec<RenderPacket> = Vec::new();
        render_packets.push(RenderPacket::Geometry(RenderPacketGeometry::new()));
        RenderList { render_packets }
    }
}

impl RenderList {
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
            _ => Err(Error::RenderList),
        }
    }

    pub fn get_render_packet_mask(&self, packet_number: usize) -> Result<&RenderPacketMask> {
        let rp = &self.render_packets[packet_number];
        match rp {
            RenderPacket::Mask(rpm) => Ok(rpm),
            _ => Err(Error::RenderList),
        }
    }

    pub fn get_render_packet_image(&self, packet_number: usize) -> Result<&RenderPacketImage> {
        let rp = &self.render_packets[packet_number];
        match rp {
            RenderPacket::Image(rpi) => Ok(rpi),
            _ => Err(Error::RenderList),
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
        let rp = self.render_packets.last_mut().ok_or(Error::RenderList)?;

        match rp {
            RenderPacket::Geometry(rpg) => {
                if !rpg.can_vertices_fit(num_vertices) {
                    if num_vertices >= RENDER_PACKET_MAX_SIZE {
                        error!(
                            "prepare_to_add_triangle_strip trying to add more than {} vertices",
                            RENDER_PACKET_MAX_SIZE
                        );
                        return Err(Error::RenderList);
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
