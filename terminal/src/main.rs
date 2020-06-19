// Copyright (C) 2020 Inderjit Gill <email@indy.io>

// This file is part of Seni

// Seni is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Seni is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use clap::{value_t, App, Arg, ArgMatches};
use config;
use env_logger;
use image::GenericImageView;
use log::{error, info, trace};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;

use core::{
    bitmaps_to_transfer, build_traits, compile_preamble, compile_program, parse,
    BitmapInfo, Context, Packable, Program, ProbeSample, VMProfiling, Var, Vm, RenderPacket,
};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

fn main() {
    let start = Instant::now();

    let matches = App::new("seni-cli")
        .version("0.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("native cli build of seni")
        .arg(
            Arg::with_name("SCRIPT")
                .help("Sets the input seni script to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .help("The seed to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("print bytecode")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("profiling")
                .short("p")
                .long("profiling")
                .help("Show opcode count")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("packed_trait_list")
                .short("t")
                .long("packed_trait_list")
                .help("print the packed trait list")
                .takes_value(false),
        )
        .get_matches();

    env_logger::init();
    if let Err(e) = run(&matches) {
        error!("Application error: {:?}", e);
    }
    let duration = start.elapsed();
    info!("Complete time elapsed: {:?}", duration);
}

fn run(matches: &ArgMatches) -> Result<()> {
    trace!("run");

    // Add in `./Settings.toml`
    // Add in settings from the environment (with a prefix of SENI)
    // Eg.. `SENI_DEBUG=1 ./target/app` would set the `debug` key
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings"))?
        .merge(config::Environment::with_prefix("SENI"))?;

    if let Some(script) = matches.value_of("SCRIPT") {
        // this should always pass as SCRIPT is required
        info!("Using script file: {}", script);

        let script = Path::new(script);

        if matches.is_present("profiling") {
            settings.set("profiling", true)?;
        }

        if matches.is_present("debug") {
            settings.set("debug", true)?;
        }

        if matches.is_present("packed_trait_list") {
            print_packed_trait_list(script)?;
        } else if let Ok(seed) = value_t!(matches.value_of("seed"), u32) {
            run_script_with_seed(script, seed, &settings)?;
        } else {
            run_script(script, &settings)?;
        }
    }

    Ok(())
}

fn read_script_file(filename: &Path) -> Result<String> {
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

    let bitmap_info = BitmapInfo::new(width, height, data);

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

fn run_script(script: &Path, settings: &config::Config) -> Result<()> {
    trace!("run_script");

    // --------------------------------------------------------------------------------

    let time_read_script_file = Instant::now();
    let source = read_script_file(script)?;

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
        let asset_prefix = settings.get_str("assets")?;
        load_bitmaps(&program, &mut context, &asset_prefix)?;

        let time_run_program = Instant::now();

        context.reset_for_piece();
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

        context.render_list.remove_useless_render_packets();
        let res = vm.top_stack_value()?;
        info!("interpret {:?}", time_interpret.elapsed());

        // vm.opcode_profiler_report();

        info!("run_program: {:?}", time_run_program.elapsed());

        if profiling == VMProfiling::On {
            vm.println_profiling(&program)?;
        }

        show_program_results(&res, &context);

        if !vm.probe_samples.is_empty() {
            print_probe_samples(&vm.probe_samples)
        }
    }

    Ok(())
}

fn print_probe_samples(probe_samples: &Vec<ProbeSample>) {
    for p in probe_samples {
        if let Some(s) = p.scalar {
            println!("fp: {fp:>width$}, sp: {sp:>width$}, ip: {ip:>width$}, scalar: {sc}", fp=p.fp, sp=p.sp, ip=p.ip, sc=s, width=3);
        } else if let Some((x,y)) = p.scalar_v2 {
            println!("fp: {fp:>width$}, sp: {sp:>width$}, ip: {ip:>width$}, v2: ({a}, {b})", fp=p.fp, sp=p.sp, ip=p.ip, a=x, b=y, width=3);
        } else {
            println!("fp: {fp:>width$}, sp: {sp:>width$}, ip: {ip:>width$}", fp=p.fp, sp=p.sp, ip=p.ip, width=3);
        }
    }
}

fn show_program_results(result: &Var, context: &Context) {
    let num_render_packets = context.render_list.get_num_render_packets();

    println!("result = {}", result);
    println!("num_render_packets: {}", num_render_packets);

    for (i, rp) in context.render_list.render_packets.iter().enumerate() {
        match rp {
            RenderPacket::Geometry(rpg) => {
                println!("{}. Geometry: {} vertices", i, rpg.geo.len());
            }
            RenderPacket::Mask(rpm) => {
                println!("{}. Mask: {}", i, rpm.filename);
            }
            RenderPacket::Image(_) => {
                println!("{}. Image", i);
            }
        }
    }
}

fn run_script_with_seed(_script: &Path, _seed: u32, _settings: &config::Config) -> Result<()> {
    trace!("run_script_with_seed");

    Ok(())
}

fn print_packed_trait_list(script: &Path) -> Result<()> {
    trace!("print_packed_trait_list");

    let source = read_script_file(script)?;
    let trait_list = build_traits(&source)?;
    let mut packed: String = "".to_string();

    trait_list.pack(&mut packed)?;
    println!("{}", packed);

    Ok(())
}
