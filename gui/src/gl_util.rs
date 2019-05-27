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

use std::mem;

use std::path::Path;

use core::BitmapInfo;
use image::GenericImageView;
use log::info;

use crate::error::Result;

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

pub fn load_texture(ppath: &Path, name: &str) -> Result<BitmapInfo> {
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

    let bitmap_info = BitmapInfo {
        width,
        height,
        data: data_flipped,
        ..Default::default()
    };

    Ok(bitmap_info)
}

pub fn identity() -> [f32; 16] {
    let out: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    out
}

pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [f32; 16] {
    let lr = 1.0 / (left - right);
    let bt = 1.0 / (bottom - top);
    let nf = 1.0 / (near - far);

    let out: [f32; 16] = [
        -2.0 * lr,
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 * bt,
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 * nf,
        0.0,
        (left + right) * lr,
        (top + bottom) * bt,
        (far + near) * nf,
        1.0,
    ];

    out
}

pub fn scale(mat: &mut [f32; 16], x: f32, y: f32, z: f32) {
    mat[0] *= x;
    mat[5] *= y;
    mat[10] *= z;
}

pub fn translate(mat: &mut [f32; 16], x: f32, y: f32, z: f32) {
    mat[12] += x;
    mat[13] += y;
    mat[14] += z;
}
