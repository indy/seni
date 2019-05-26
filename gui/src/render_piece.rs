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

use gl::types::*;

use std::path::Path;

use core::Geometry;

use crate::error::Result;
use crate::gl_util;
use crate::render_gl;

pub struct Renderer {
    gl: gl::Gl,
    program: render_gl::Program,

    vao: GLuint, // todo: render_imgui recreates this on every call to render. why???
    vbo: GLuint,

    texture: GLuint,
    // locations: RendererLocations,
}

struct RendererLocations {
    texture: GLint,
    projection_mtx: GLint,
    modelview_mtx: GLint,

    position: GLuint,
    colour: GLuint,
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
    pub fn new(gl: &gl::Gl, assets_path: &Path, bitmaps_path: &Path) -> Result<Renderer> {
        let program = render_gl::Program::from_path(gl, assets_path, "shaders/triangle")?;
        let bitmap_info = gl_util::load_texture(&bitmaps_path, "texture.png")?;

        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let mut texture: gl::types::GLuint = 0;

        let location_texture: gl::types::GLint;
        let location_projection_mtx: gl::types::GLint;
        let location_modelview_mtx: gl::types::GLint;
        let location_position: gl::types::GLuint;
        let location_colour: gl::types::GLuint;
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
            location_colour = gl.GetAttribLocation(program.id(), c_str!("Colour").as_ptr()) as _;
            location_uv = gl.GetAttribLocation(program.id(), c_str!("UV").as_ptr()) as _;

            gl.GenTextures(1, &mut texture);
            // "Bind" the newly created texture : all future texture functions will modify this texture
            gl.BindTexture(gl::TEXTURE_2D, texture);

            // Give the image to OpenGL
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                bitmap_info.width as i32,
                bitmap_info.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                bitmap_info.data.as_ptr() as *const gl::types::GLvoid,
            );

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
            colour: location_colour,
            uv: location_uv,
        };

        let projection_matrix = gl_util::ortho(0.0, 1920.0, 0.0, 1062.0, 10.0, -10.0);
        let model_view_matrix = gl_util::identity();

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
            gl.EnableVertexAttribArray(locations.colour);
            gl.EnableVertexAttribArray(locations.uv);

            gl.VertexAttribPointer(
                locations.position,
                2,         // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null(), // offset of the first component
            );
            gl.VertexAttribPointer(
                locations.colour, // index of the generic vertex attribute ("layout (location = 1)")
                4,                // the number of components per generic vertex attribute
                gl::FLOAT,        // data type
                gl::FALSE,        // normalized (int-to-float conversion)
                (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
            );
            gl.VertexAttribPointer(
                locations.uv, // index of the generic vertex attribute ("layout (location = 2)")
                2,            // the number of components per generic vertex attribute
                gl::FLOAT,    // data type
                gl::FALSE,    // normalized (int-to-float conversion)
                (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
            );
        }

        Ok(Renderer {
            gl: gl.clone(),
            program,
            vao,
            vbo,
            texture,
            // locations,
        })
    }

    pub fn render(&self, geometry: &Geometry) {
        let gl = &self.gl;

        unsafe {
            gl.ActiveTexture(gl::TEXTURE0);
            gl.UseProgram(self.program.id());

            gl.BindVertexArray(self.vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            for rp in &geometry.render_packets {
                gl.BufferData(
                    gl::ARRAY_BUFFER,                                                     // target
                    (rp.geo.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                    rp.geo.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::STATIC_DRAW,                             // usage
                );

                gl.DrawArrays(
                    gl::TRIANGLE_STRIP,  // mode
                    0,                   // starting index in the enabled arrays
                    rp.geo.len() as i32, // number of indices to be rendered
                );
            }
        }
    }
}
