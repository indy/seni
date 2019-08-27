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

use std::mem;

use std::path::Path;

use gl;
use image::GenericImageView;
use log::info;

use crate::error::{Error, Result};



#[derive(Default)]
pub struct BitmapU8Info {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[macro_export]
macro_rules! c_str {
    ($literal:expr) => {
        std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    };
}

pub fn return_param<T, F>(f: F) -> T
where
    F: FnOnce(&mut T),
{
    let mut val = unsafe { mem::uninitialized() };
    f(&mut val);
    val
}

pub fn load_texture(ppath: &Path, name: &str) -> Result<BitmapU8Info> {
    let path = ppath.join(name);

    info!("load_bitmap: {:?}", path);
    let image = image::open(&path)?;

    let (w, h) = image.dimensions();
    let width = w as usize;
    let height = h as usize;
    let mut data: Vec<u8> = Vec::with_capacity(width * height * 4);

    info!("loading bitmap {} of size {} x {}", name, width, height);

    for (_, _, rgba) in image.pixels() {
        data.push(rgba.data[0]);
        data.push(rgba.data[1]);
        data.push(rgba.data[2]);
        data.push(rgba.data[3]);
    }

    let mut data_flipped: Vec<u8> = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let offset = ((height - y - 1) * (width * 4)) + (x * 4);
            data_flipped.push(data[offset]);
            data_flipped.push(data[offset + 1]);
            data_flipped.push(data[offset + 2]);
            data_flipped.push(data[offset + 3]);
        }
    }

    let bitmap_info = BitmapU8Info {
        width,
        height,
        data: data_flipped,
        ..Default::default()
    };

    Ok(bitmap_info)
}

pub fn create_framebuffer(gl: &gl::Gl) -> gl::types::GLuint {
    let mut framebuffer_id: gl::types::GLuint = 0;

    unsafe {
        gl.GenFramebuffers(1, &mut framebuffer_id);
    }

    framebuffer_id
}

pub fn create_texture(gl: &gl::Gl, width: usize, height: usize) -> gl::types::GLuint {
    let mut texture_id: gl::types::GLuint = 0;

    unsafe {
        gl.GenTextures(1, &mut texture_id);
        // "Bind" the newly created texture : all future texture functions will modify this texture
        gl.BindTexture(gl::TEXTURE_2D, texture_id);
        // Give an empty image to OpenGL ( the last "0" )
        gl.TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as _,
            width as _,
            height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            0 as *const gl::types::GLvoid,
        );

        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

        gl.BindTexture(gl::TEXTURE_2D, 0);
    }

    texture_id
}

pub fn delete_texture(gl: &gl::Gl, id: gl::types::GLuint) {
    // todo: is this the right way of deleting textures? needs testing
    let u = [id].as_ptr();

    unsafe {
        gl.DeleteTextures(1, u as *const u32);
    }
}

pub fn attach_texture_to_framebuffer(
    gl: &gl::Gl,
    framebuffer_id: gl::types::GLuint,
    texture_id: gl::types::GLuint,
) {
    unsafe {
        gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
        gl.FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture_id,
            0,
        );
    }
}

pub fn is_framebuffer_ok(gl: &gl::Gl) -> Result<()> {
    unsafe {
        if gl.CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            Err(Error::GLError("Framebuffer is not complete".to_string()))
        } else {
            Ok(())
        }
    }
}

pub fn bind_framebuffer(
    gl: &gl::Gl,
    framebuffer_id: gl::types::GLuint,
    viewport_width: usize,
    viewport_height: usize,
) {
    unsafe {
        gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
        gl.Viewport(0, 0, viewport_width as _, viewport_height as _);
    }
}

pub fn update_viewport(gl: &gl::Gl, viewport_width: usize, viewport_height: usize) {
    unsafe {
        gl.Viewport(0, 0, viewport_width as _, viewport_height as _);
    }
}
