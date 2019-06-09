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

use std::path::Path;

use core::Context;
use gl;

use crate::error::Result;
use crate::gl_util;
use crate::render_piece;
use crate::render_square;

pub struct Renderer {
    piece_renderer: render_piece::Renderer,
    square_renderer: render_square::Renderer,
    render_texture_id: gl::types::GLuint,
    framebuffer_id: gl::types::GLuint,
}

impl Renderer {
    pub fn new(
        gl: &gl::Gl,
        assets_path: &Path,
        bitmaps_path: &Path,
        context: &Context,
    ) -> Result<Renderer> {
        let piece_renderer = render_piece::Renderer::new(&gl, &assets_path, &bitmaps_path)?;

        let render_texture_id = gl_util::create_texture(&gl, 1024, 1024);
        let framebuffer_id = gl_util::create_framebuffer(&gl);
        gl_util::attach_texture_to_framebuffer(&gl, framebuffer_id, render_texture_id);
        gl_util::is_framebuffer_ok(&gl)?;
        gl_util::bind_framebuffer(&gl, framebuffer_id, 1024, 1024);

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        piece_renderer.render(&context.geometry, 1000, 1000);

        // putting in nonsense viewport figures since a gl_util::upate_viewport is called after this constructor
        gl_util::bind_framebuffer(&gl, 0, 1, 1);

        let square_renderer = render_square::Renderer::new(&gl, &assets_path, render_texture_id)?;

        Ok(Renderer {
            piece_renderer,
            square_renderer,
            render_texture_id,
            framebuffer_id,
        })
    }

    pub fn rebake(
        &mut self,
        gl: &gl::Gl,
        context: &Context,
        viewport_width: usize,
        viewport_height: usize,
    ) -> Result<()> {
        gl_util::bind_framebuffer(&gl, self.framebuffer_id, 1024, 1024);

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.piece_renderer.render(&context.geometry, 1000, 1000);

        // bind back to the default rendering output
        gl_util::bind_framebuffer(&gl, 0, viewport_width, viewport_height);

        Ok(())
    }

    pub fn render(&self, viewport_width: usize, viewport_height: usize) {
        self.square_renderer.render(viewport_width, viewport_height);
    }
}
