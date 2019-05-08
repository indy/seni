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

use crate::context::Context;
use crate::error::Error;
use crate::iname::Iname;
use crate::keywords::Keyword;
use crate::program::Program;
use crate::result::Result;
use crate::vm::Vm;

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

    let e0 = bitmap_info.data[index];
    let e1 = bitmap_info.data[index + 1];
    let e2 = bitmap_info.data[index + 2];
    let e3 = bitmap_info.data[index + 2];

    vm.function_call_default_arguments(context, program, fn_info)?;
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::A), e0 as f32 / 255.0);
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::B), e1 as f32 / 255.0);
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::C), e2 as f32 / 255.0);
    vm.function_set_argument_to_f32(fn_info, Iname::from(Keyword::D), e3 as f32 / 255.0);
    vm.function_set_argument_to_2d(fn_info, Iname::from(Keyword::Position), x as f32, y as f32);
    vm.function_call_body(context, program, fn_info)?;

    vm.ip = ip;

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
) -> Result<()> {
    // get the bitmap from the context
    // hardcoded: normally use 'from' to lookup string value in program's data struct
    // let bitmap_info = context.bitmap_cache.get("img/bitmap1.png")?;

    let from_string = if let Some(from_string) = program.data.strings.get(&from) {
        //info!("found string: {}", from_string);
        from_string
    } else {
        return Err(Error::Bitmap(format!(
            "unable to find string from iname: {}",
            from
        )));
    };

    let (width, height) = {
        let bitmap_info = context.bitmap_cache.get(from_string)?;
        (bitmap_info.width, bitmap_info.height)
    };

    let tx = dst_width / width as f32;
    let ty = dst_height / height as f32;

    let originx = dst_position.0 - (dst_width / 2.0);
    let originy = dst_position.1 - (dst_height / 2.0);

    for y in 0..height {
        for x in 0..width {
            // setup matrix stack
            context.matrix_stack.push();
            {
                //  origin  + pixel location  + offset to center each pixel
                context.matrix_stack.translate(
                    originx + (tx * x as f32) + (tx / 2.0),
                    originy + (ty * y as f32) + (ty / 2.0),
                );
                context.matrix_stack.scale(tx, ty);

                // assuming that the bitmap is in u8rgba format
                let index = ((height - y - 1) * width * 4) + (x * 4);

                invoke_function(vm, context, program, fun, x, y, index, from_string)?;
            }
            context.matrix_stack.pop();
        }
    }

    Ok(())
}
