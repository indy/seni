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

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

use config;
use core::{
    bitmaps_to_transfer, compile_preamble, compile_program, parse, BitmapInfo, Context, Program,
    VMProfiling, Vm,
};

use image::GenericImageView;
use log::{info, trace};

use crate::error::Result;

fn load_bitmap(asset_prefix: &String, filename: &String, context: &mut Context) -> Result<()> {
    let path = Path::new(asset_prefix).join(filename);
    info!("load_bitmap: {:?}", path);
    let image = image::open(&path)?;

    let (w, h) = image.dimensions();
    let width = w as usize;
    let height = h as usize;
    let mut data: Vec<u8> = Vec::with_capacity(width * height * 4);

    info!("loading bitmap {} of size {} x {}", filename, width, height);

    for (_, _, rgba) in image.pixels() {
        data.push(rgba.data[0]);
        data.push(rgba.data[1]);
        data.push(rgba.data[2]);
        data.push(rgba.data[3]);
    }

    let bitmap_info = BitmapInfo {
        width,
        height,
        data,
        ..Default::default()
    };

    context.bitmap_cache.insert(&filename, bitmap_info)?;

    Ok(())
}

fn load_bitmaps(program: &Program, context: &mut Context, asset_prefix: &String) -> Result<()> {
    let time_to_load_bitmaps = Instant::now();

    let bitmaps_to_transfer = bitmaps_to_transfer(&program, &context);
    let len = bitmaps_to_transfer.len();

    if len == 0 {
        return Ok(());
    }

    for f in bitmaps_to_transfer {
        load_bitmap(asset_prefix, &f, context)?;
    }

    info!(
        "loading {}: {:?}",
        quantity(len, "bitmap"),
        time_to_load_bitmaps.elapsed()
    );

    Ok(())
}

fn read_script_file(filename: &PathBuf) -> Result<String> {
    trace!("read_script_file");

    let mut f = File::open(filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(contents)
}

fn quantity(amount: usize, s: &str) -> String {
    if amount == 1 {
        return format!("{} {}", amount, s);
    } else {
        return format!("{} {}s", amount, s);
    }
}

pub fn run_script(script: &str, settings: &config::Config) -> Result<Context> {
    trace!("run_script");

    // --------------------------------------------------------------------------------

    let time_read_script_file = Instant::now();
    let scripts_directory = settings.get_str("scripts_path")?;
    let source = read_script_file(&Path::new(&scripts_directory).join(script))?;

    info!("read_script_file: {:?}", time_read_script_file.elapsed());

    let mut vm: Vm = Default::default();
    let mut context: Context = Default::default();

    // --------------------------------------------------------------------------------

    let time_parse = Instant::now();
    let (ast, word_lut) = parse(&source)?;
    info!("parse: {:?}", time_parse.elapsed());

    // --------------------------------------------------------------------------------

    let time_compile_program = Instant::now();
    let program = compile_program(&ast, &word_lut)?;
    info!("compile_program: {:?}", time_compile_program.elapsed());

    // --------------------------------------------------------------------------------

    if settings.get_bool("debug")? {
        // print the source and bytecode without trying to run the code
        // as the debug option will often be used with buggy source
        println!("{}", source);
        println!("{}", program);
    } else {
        let bitmap_prefix = settings.get_str("bitmaps_path")?;
        load_bitmaps(&program, &mut context, &bitmap_prefix)?;

        let time_run_program = Instant::now();

        context.reset();
        vm.reset();

        // setup the env with the global variables in preamble
        let time_preamble = Instant::now();
        let preamble = compile_preamble()?;
        vm.interpret(&mut context, &preamble)?;
        info!("preamble: {:?}", time_preamble.elapsed());

        // reset the ip and setup any profiling of the main program
        let profiling = if settings.get_bool("profiling")? {
            VMProfiling::On
        } else {
            VMProfiling::Off
        };
        vm.init_for_main_program(&program, profiling)?;

        let time_interpret = Instant::now();
        vm.interpret(&mut context, &program)?;
        let res = vm.top_stack_value()?;
        info!("interpret {:?}", time_interpret.elapsed());

        // vm.opcode_profiler_report();

        info!("run_program: {:?}", time_run_program.elapsed());

        if profiling == VMProfiling::On {
            vm.println_profiling(&program)?;
        }

        println!("res = {}", res);
    }

    Ok(context)
}
