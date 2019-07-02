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

use crate::bitmap_cache::BitmapCache;
use crate::colour::Colour;
use crate::ease::Easing;
use crate::error::Error;
use crate::geometry;
use crate::matrix::MatrixStack;
use crate::result::Result;
use crate::rgb::Rgb;
use crate::uvmapper::{BrushType, Mappings};
use crate::vm::Var;

use log::error;

#[derive(Default)]
pub struct Context {
    pub matrix_stack: MatrixStack,
    pub mappings: Mappings,
    pub geometry: geometry::Geometry,
    pub bitmap_cache: BitmapCache,
    pub output_linear_colour_space: bool, // derive Default sets bool to false
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
            let from_col = Rgb::from_colour(from_col)?;
            let to_col = Rgb::from_colour(to_col)?;

            geometry::line::render(
                &mut self.geometry,
                matrix,
                from,
                to,
                width,
                &from_col,
                &to_col,
                uvm,
            )
        } else {
            error!("no matrix for render_line");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::rect::render(
                &mut self.geometry,
                matrix,
                position,
                width,
                height,
                &colour,
                uvm,
            )
        } else {
            error!("no matrix for render_rect");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::circle::render(
                &mut self.geometry,
                matrix,
                position,
                width,
                height,
                &colour,
                tessellation,
                uvm,
            )
        } else {
            error!("no matrix for render_circle");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::circle_slice::render(
                &mut self.geometry,
                matrix,
                position,
                width,
                height,
                &colour,
                tessellation,
                angle_start,
                angle_end,
                inner_width,
                inner_height,
                uvm,
            )
        } else {
            error!("no matrix for render_circle_slice");
            Err(Error::Context)
        }
    }

    pub fn render_poly(&mut self, coords: &[Var], colours: &[Var]) -> Result<()> {
        let coords: Result<Vec<(f32, f32)>> =
            coords.into_iter().map(|c| var_to_f32_pair(c)).collect();
        let coords = coords?;

        let colours: Result<Vec<Rgb>> = colours.into_iter().map(|c| var_to_rgb(c)).collect();
        let colours = colours?;

        if let Some(matrix) = self.matrix_stack.peek() {
            let uvm = self.mappings.get_uv_mapping(BrushType::Flat, 0);
            geometry::poly::render(&mut self.geometry, matrix, &coords, &colours, uvm)
        } else {
            error!("no matrix for render_poly");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::quadratic::render(
                &mut self.geometry,
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                &colour,
                tessellation,
                uvm,
            )
        } else {
            error!("no matrix for render_quadratic");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::bezier::render(
                &mut self.geometry,
                matrix,
                coords,
                width_start,
                width_end,
                width_mapping,
                t_start,
                t_end,
                &colour,
                tessellation,
                uvm,
            )
        } else {
            error!("no matrix for render_bezier");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::bezier_bulging::render(
                &mut self.geometry,
                matrix,
                coords,
                line_width,
                t_start,
                t_end,
                &colour,
                tessellation,
                uvm,
            )
        } else {
            error!("no matrix for render_bezier_bulging");
            Err(Error::Context)
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
            let colour = Rgb::from_colour(colour)?;

            geometry::stroked_bezier::render(
                &mut self.geometry,
                matrix,
                tessellation,
                coords,
                stroke_tessellation,
                stroke_noise,
                stroke_line_width_start,
                stroke_line_width_end,
                &colour,
                colour_volatility,
                seed,
                mapping,
                uvm,
            )
        } else {
            error!("no matrix for render_stroked_bezier");
            Err(Error::Context)
        }
    }
}

fn var_to_f32_pair(v: &Var) -> Result<(f32, f32)> {
    if let Var::V2D(x, y) = v {
        Ok((*x, *y))
    } else {
        error!("var_to_f32_pair");
        Err(Error::Context)
    }
}

fn var_to_rgb(v: &Var) -> Result<(Rgb)> {
    if let Var::Colour(col) = v {
        let rgb = Rgb::from_colour(&col)?;
        Ok(rgb)
    } else {
        error!("var_to_rgb");
        Err(Error::Context)
    }
}
