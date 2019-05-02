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

use crate::colour::Colour;
use crate::ease::Easing;
use crate::error::Error;
use crate::geometry::Geometry;
use crate::matrix::MatrixStack;
use crate::result::Result;
use crate::uvmapper::{BrushType, Mappings};
use crate::vm::Var;

#[derive(Default)]
pub struct Context {
    pub matrix_stack: MatrixStack,
    pub mappings: Mappings,
    pub geometry: Geometry,
}

impl Context {
    pub fn reset(&mut self) {
        self.matrix_stack.reset();
        self.geometry.reset();
    }

    pub fn get_render_packet_geo_len(&self, packet_number: usize) -> usize {
        self.geometry.get_render_packet_geo_len(packet_number)
    }

    pub fn get_render_packet_geo_ptr(&self, packet_number: usize) -> *const f32 {
        self.geometry.get_render_packet_geo_ptr(packet_number)
    }

    pub fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        width: f32,
        from_col: &Colour,
        to_col: &Colour,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry
                .render_line(matrix, from, to, width, from_col, to_col, uvm)
        } else {
            Err(Error::VM("no matrix for render_line".to_string()))
        }
    }
    pub fn render_rect(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry
                .render_rect(matrix, position, width, height, colour, uvm)
        } else {
            Err(Error::VM("no matrix for render_rect".to_string()))
        }
    }

    pub fn render_circle(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry
                .render_circle(matrix, position, width, height, colour, tessellation, uvm)
        } else {
            Err(Error::VM("no matrix for render_circle".to_string()))
        }
    }

    pub fn render_circle_slice(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        colour: &Colour,
        tessellation: usize,
        angle_start: f32,
        angle_end: f32,
        inner_width: f32,
        inner_height: f32,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry.render_circle_slice(
                matrix,
                position,
                width,
                height,
                colour,
                tessellation,
                angle_start,
                angle_end,
                inner_width,
                inner_height,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_circle_slice".to_string()))
        }
    }

    pub fn render_poly(&mut self, coords: &[Var], colours: &[Var]) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            self.geometry.render_poly(matrix, coords, colours, uvm)
        } else {
            Err(Error::VM("no matrix for render_poly".to_string()))
        }
    }

    pub fn render_quadratic(
        &mut self,
        coords: &[f32; 6],
        width_start: f32,
        width_end: f32,
        width_mapping: Easing,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_quadratic(
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_quadratic".to_string()))
        }
    }

    pub fn render_bezier(
        &mut self,
        coords: &[f32; 8],
        width_start: f32,
        width_end: f32,
        width_mapping: Easing,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_bezier(
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_bezier".to_string()))
        }
    }

    pub fn render_bezier_bulging(
        &mut self,
        coords: &[f32; 8],
        line_width: f32,
        t_start: f32,
        t_end: f32,
        colour: &Colour,
        tessellation: usize,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_bezier_bulging(
                matrix,
                coords,
                line_width,
                t_start,
                t_end,
                colour,
                tessellation,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_bezier_bulging".to_string()))
        }
    }

    pub fn render_stroked_bezier(
        &mut self,
        tessellation: usize,
        coords: &[f32; 8],
        stroke_tessellation: usize,
        stroke_noise: f32,
        stroke_line_width_start: f32,
        stroke_line_width_end: f32,
        colour: &Colour,
        colour_volatility: f32,
        seed: f32,
        mapping: Easing,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_stroked_bezier(
                matrix,
                tessellation,
                coords,
                stroke_tessellation,
                stroke_noise,
                stroke_line_width_start,
                stroke_line_width_end,
                colour,
                colour_volatility,
                seed,
                mapping,
                uvm,
            )
        } else {
            Err(Error::VM("no matrix for render_stroked_bezier".to_string()))
        }
    }

    pub fn render_stroked_bezier_rect(
        &mut self,
        position: (f32, f32),
        width: f32,
        height: f32,
        volatility: f32,
        overlap: f32,
        iterations: f32,
        seed: i32,
        tessellation: usize,
        stroke_tessellation: usize,
        stroke_noise: f32,
        colour: &Colour,
        colour_volatility: f32,
        brush_type: BrushType,
        brush_subtype: usize,
    ) -> Result<()> {
        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(brush_type, brush_subtype);

            self.geometry.render_stroked_bezier_rect(
                matrix,
                position,
                width,
                height,
                volatility,
                overlap,
                iterations,
                seed,
                tessellation,
                stroke_tessellation,
                stroke_noise,
                colour,
                colour_volatility,
                uvm,
            )
        } else {
            Err(Error::VM(
                "no matrix for render_stroked_bezier_rect".to_string(),
            ))
        }
    }
}
