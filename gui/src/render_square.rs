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

use gl::types::*;

use std::path::Path;

use crate::error::Result;
use crate::matrix_util;
use crate::render_gl;

struct GeometryLayout {
    stride: usize,
    position_num_elements: usize,
    texture_num_elements: usize,
}

pub struct Renderer {
    gl: gl::Gl,
    program: render_gl::Program,

    vao: GLuint,
    vbo: GLuint,

    texture: GLuint,

    geometry: Vec<f32>,

    locations: RendererLocations,
}

struct RendererLocations {
    texture: GLint,
    projection_mtx: GLint,
    modelview_mtx: GLint,

    position: GLuint,
    uv: GLuint,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let gl = &self.gl;

        // todo: should program be explicitly dropped or does that happen implicitly?
        unsafe {
            gl.DeleteBuffers(1, &self.vbo);
            gl.DeleteTextures(1, &self.texture);
            gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

impl Renderer {
    pub fn new(gl: &gl::Gl, assets_path: &Path, texture: gl::types::GLuint) -> Result<Renderer> {
        let program = render_gl::Program::from_path(gl, assets_path, "shaders/blit")?;

        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;

        let location_texture: gl::types::GLint;
        let location_projection_mtx: gl::types::GLint;
        let location_modelview_mtx: gl::types::GLint;
        let location_position: gl::types::GLuint;
        let location_uv: gl::types::GLuint;

        program.set_used();

        unsafe {
            // set up vertex array object
            //
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            // set up vertex buffer object
            //
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

            location_texture =
                gl.GetUniformLocation(program.id(), c_str!("myTextureSampler").as_ptr());
            location_projection_mtx =
                gl.GetUniformLocation(program.id(), c_str!("uPMatrix").as_ptr());
            location_modelview_mtx =
                gl.GetUniformLocation(program.id(), c_str!("uMVMatrix").as_ptr());

            location_position =
                gl.GetAttribLocation(program.id(), c_str!("Position").as_ptr()) as _;
            location_uv = gl.GetAttribLocation(program.id(), c_str!("UV").as_ptr()) as _;

            gl.BindTexture(gl::TEXTURE_2D, texture);

            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl.TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.GenerateMipmap(gl::TEXTURE_2D);
        }

        let locations = RendererLocations {
            texture: location_texture,
            projection_mtx: location_projection_mtx,
            modelview_mtx: location_modelview_mtx,
            position: location_position,
            uv: location_uv,
        };

        let projection_matrix =
            matrix_util::create_ortho_matrix(0.0, 1000.0, 0.0, 1000.0, 10.0, -10.0);
        let mut model_view_matrix = matrix_util::create_identity_matrix();
        matrix_util::matrix_scale(&mut model_view_matrix, 800.0, 800.0, 1.0);
        matrix_util::matrix_translate(&mut model_view_matrix, 100.0, 100.0, 0.0);

        let layout = GeometryLayout {
            stride: 4, // x, y, u, v
            position_num_elements: 2,
            texture_num_elements: 2,
        };

        unsafe {
            gl.Uniform1i(locations.texture, 0);
            gl.UniformMatrix4fv(
                locations.projection_mtx,
                1,
                gl::FALSE,
                projection_matrix.as_ptr(),
            );
            gl.UniformMatrix4fv(
                locations.modelview_mtx,
                1,
                gl::FALSE,
                model_view_matrix.as_ptr(),
            );

            gl.EnableVertexAttribArray(locations.position);
            gl.EnableVertexAttribArray(locations.uv);

            gl.VertexAttribPointer(
                locations.position,
                layout.position_num_elements as i32, // the number of components per generic vertex attribute
                gl::FLOAT,                           // data type
                gl::FALSE,                           // normalized (int-to-float conversion)
                (layout.stride * std::mem::size_of::<f32>()) as gl::types::GLint, // stride
                std::ptr::null(),                    // offset of the first component
            );

            let texture_offset = layout.position_num_elements;
            gl.VertexAttribPointer(
                locations.uv,
                layout.texture_num_elements as i32, // the number of components per generic vertex attribute
                gl::FLOAT,                          // data type
                gl::FALSE,                          // normalized (int-to-float conversion)
                (layout.stride * std::mem::size_of::<f32>()) as gl::types::GLint, // stride
                (texture_offset * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
            );
        }

        // generate geometry
        let geometry: Vec<f32> = vec![
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];

        unsafe {
            gl.BufferData(
                gl::ARRAY_BUFFER,                                                       // target
                (geometry.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                geometry.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW,                               // usage
            );
        }

        Ok(Renderer {
            gl: gl.clone(),
            program,
            vao,
            vbo,
            texture,
            geometry,
            locations,
        })
    }

    pub fn render(&self, viewport_width: usize, viewport_height: usize) {
        let gl = &self.gl;

        let projection_matrix = matrix_util::create_ortho_matrix(
            0.0,
            viewport_width as f32,
            0.0,
            viewport_height as f32,
            10.0,
            -10.0,
        );

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0);
            gl.BindTexture(gl::TEXTURE_2D, self.texture);

            gl.UseProgram(self.program.id());

            gl.BindVertexArray(self.vao);

            gl.UniformMatrix4fv(
                self.locations.projection_mtx,
                1,
                gl::FALSE,
                projection_matrix.as_ptr(),
            );

            gl.DrawArrays(
                gl::TRIANGLE_STRIP,         // mode
                0,                          // starting index in the enabled arrays
                self.geometry.len() as i32, // number of indices to be rendered
            );
        }
    }
}
