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

use crate::colour::{Colour, ColourFormat};
use crate::context::Context;
use crate::error::Error;
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::prng::PrngStateStruct;
use crate::program::Program;
use crate::result::Result;
use crate::vm::Vm;

use log::error;
use rand::seq::SliceRandom;

// invoke a function with args: x, y, r, g, b, a
// colour values are normalized to 0..1
fn invoke_function(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    x: usize,
    y: usize,
    index: usize,
    from_string: &str,
) -> Result<()> {
    let bitmap_info = context.bitmap_cache.get(from_string)?;
    let ip = vm.ip;
    let fn_info = &program.fn_info[fun];
    let colour = Colour::new(
        ColourFormat::Rgb,
        f32::from(bitmap_info.data[index]) / 255.0,
        f32::from(bitmap_info.data[index + 1]) / 255.0,
        f32::from(bitmap_info.data[index + 2]) / 255.0,
        f32::from(bitmap_info.data[index + 3]) / 255.0,
    );

    vm.function_call_default_arguments(context, program, fn_info)?;
    vm.function_set_argument_to_col(fn_info, Iname::from(Keyword::Colour), &colour);
    vm.function_set_argument_to_2d(fn_info, Iname::from(Keyword::Position), x as f32, y as f32);
    vm.function_call_body(context, program, fn_info)?;

    vm.ip = ip;

    Ok(())
}

fn per_pixel(
    pos: (usize, usize),
    context: &mut Context,
    bitmap_dim: (usize, usize),
    origin: (f32, f32),
    scale_factor: (f32, f32),
    vm: &mut Vm,
    program: &Program,
    fun: usize,
    from_string: &String,
) -> Result<()> {
    // setup matrix stack
    context.matrix_stack.push();
    {
        //  origin  + pixel location  + offset to center each pixel
        context.matrix_stack.translate(
            origin.0 + (scale_factor.0 * pos.0 as f32) + (scale_factor.0 / 2.0),
            origin.1 + (scale_factor.1 * pos.1 as f32) + (scale_factor.1 / 2.0),
        );
        context.matrix_stack.scale(scale_factor.0, scale_factor.1);

        // assuming that the bitmap is in u8rgba format
        let index = ((bitmap_dim.1 - pos.1 - 1) * bitmap_dim.0 * 4) + (pos.0 * 4);

        invoke_function(vm, context, program, fun, pos.0, pos.1, index, from_string)?;
    }
    context.matrix_stack.pop();

    Ok(())
}

pub fn each(
    vm: &mut Vm,
    context: &mut Context,
    program: &Program,
    fun: usize,
    from: Iname,
    dst_position: (f32, f32),
    dst_width: f32,
    dst_height: f32,
    shuffle_seed: Option<f32>,
) -> Result<()> {
    let from_string = if let Some(from_string) = program.data.strings.get(&from) {
        from_string
    } else {
        error!("unable to find string from iname: {}", from);
        return Err(Error::Bitmap);
    };

    let bitmap_dim = {
        let bitmap_info = context.bitmap_cache.get(from_string)?;
        (bitmap_info.width, bitmap_info.height)
    };

    let scale_factor: (f32, f32) = (
        dst_width / bitmap_dim.0 as f32,
        dst_height / bitmap_dim.1 as f32,
    );

    let origin: (f32, f32) = (
        dst_position.0 - (dst_width / 2.0),
        dst_position.1 - (dst_height / 2.0),
    );

    if let Some(seed) = shuffle_seed {
        let mut prng = PrngStateStruct::new(seed as i32, 0.0, 1.0);
        let mut coords: Vec<(usize, usize)> = Vec::with_capacity(bitmap_dim.0 * bitmap_dim.1);

        for y in 0..bitmap_dim.1 {
            for x in 0..bitmap_dim.0 {
                coords.push((x, y))
            }
        }
        // using rand crate's shuffle, if this is too expensive write my own version
        coords.shuffle(&mut prng.rng);

        for coord in coords {
            per_pixel(
                coord,
                context,
                bitmap_dim,
                origin,
                scale_factor,
                vm,
                program,
                fun,
                from_string,
            )?;
        }
    } else {
        for y in 0..bitmap_dim.1 {
            for x in 0..bitmap_dim.0 {
                per_pixel(
                    (x, y),
                    context,
                    bitmap_dim,
                    origin,
                    scale_factor,
                    vm,
                    program,
                    fun,
                    from_string,
                )?;
            }
        }
    }

    Ok(())
}
