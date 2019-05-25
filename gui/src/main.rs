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

mod error;
mod render_gl;

use std::path::{Path, PathBuf};
use std::time::Instant;
use std::ffi;
use std::fs::File;
use std::io::prelude::*;

use clap::{value_t, App, Arg};
use config;
use env_logger;
use image::GenericImageView;
use log::{info, trace};
use core::{
    bitmaps_to_transfer, compile_preamble, compile_program, parse, BitmapInfo,
    Context, Program, VMProfiling, Vm,
};

use gl;
use imgui;
use imgui_opengl_renderer;
use imgui_sdl2;
use sdl2;

use crate::error::Result;

#[macro_export]
macro_rules! c_str {
    ($literal:expr) => {
        ffi::CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
    };
}

fn main() -> Result<()> {
    // Add in `./Config.toml`
    // Add in config from the environment (with a prefix of SENI)
    // Eg.. `SENI_DEBUG=1 ./target/app` would set the `debug` key
    //
    let mut config = config::Config::default();
    config
        .merge(config::File::with_name("Config"))?
        .merge(config::Environment::with_prefix("SENI"))?;

    // update config with command line options
    //
    let matches = App::new("seni-gui")
        .version("4.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("native gui build of seni")
        .arg(
            Arg::with_name("SCRIPT")
                .help("Sets the input seni script to use")
                .index(1),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .help("The seed to use")
                .takes_value(true),
        )
        .get_matches();

    env_logger::init();

    if let Some(script) = matches.value_of("SCRIPT") {
        // this should always pass as SCRIPT is required
        info!("Using script file: {}", script);

        config.set("script", script)?;
    }

    if let Ok(seed) = value_t!(matches.value_of("seed"), i64) {
        config.set("seed", seed)?;
    }

    run(&config)
}

fn run(config: &config::Config) -> Result<()> {
    let assets_location = config.get_str("assets_path")?;
    let assets_path = Path::new(&assets_location);

    let bitmap_location = config.get_str("bitmaps_path")?;
    let bitmap_path = Path::new(&bitmap_location);

    let script_filename = config.get_str("script")?;

    let seni_context = run_script(&script_filename, &config)?;

    let bitmap_info = load_texture(&bitmap_path, "texture.png")?;

    let projection_matrix = ortho(0.0, 1920.0, 0.0, 1062.0, 10.0, -10.0);
    // let projection_matrix = ortho(0.0, 1000.0, 0.0, 1000.0, 10.0, -10.0);
    let model_view_matrix = identity();

    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;

    {
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(4, 5);
    }

    let window = video
        .window("seni-gui", 1000, 1000)
        .position_centered()
        .resizable()
        .opengl()
        .allow_highdpi()
        .build()?;

    let _gl_context = window
        .gl_create_context()
        .expect("Couldn't create GL context");
    // provide a function to load function pointer by string
    let gl = gl::Gl::load_with(|s| video.gl_get_proc_address(s) as _);

    let mut imgui = imgui::ImGui::init();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui);

    let renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| video.gl_get_proc_address(s) as _);

    // --------------------------------------------------------------------------------
    // set up shader program
    //
    let shader_program = render_gl::Program::from_path(&gl, &assets_path, "shaders/triangle")?;

    // set up vertex buffer object
    //

    let mut vbo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;
    let mut tex: gl::types::GLuint = 0;

    let gl_int_p_matrix: gl::types::GLint;
    let gl_int_mv_matrix: gl::types::GLint;

    shader_program.set_used();
    unsafe {
        // set up vertex buffer object
        //
        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,                                                       // target
            (seni_context.geometry.render_packets[0].geo.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            seni_context.geometry.render_packets[0].geo.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW,                               // usage
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);

        // set up vertex array object
        //
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl.VertexAttribPointer(
            0,         // index of the generic vertex attribute ("layout (location = 0)")
            2,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(),                                     // offset of the first component
        );
        gl.EnableVertexAttribArray(1); // this is "layout (location = 1)" in vertex shader
        gl.VertexAttribPointer(
            1,         // index of the generic vertex attribute ("layout (location = 1)")
            4,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );
        gl.EnableVertexAttribArray(2); // this is "layout (location = 2)" in vertex shader
        gl.VertexAttribPointer(
            2,         // index of the generic vertex attribute ("layout (location = 2)")
            2,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (8 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );

        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);

        // set up shared state for window
        //
        gl.Viewport(0, 0, 900, 700);
        gl.ClearColor(1.0, 1.0, 1.0, 1.0);

        // assuming that we'll be using pre-multiplied alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        gl.Enable(gl::BLEND);
        gl.BlendEquation(gl::FUNC_ADD);
        gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);

        gl.GenTextures(1, &mut tex);
        // "Bind" the newly created texture : all future texture functions will modify this texture
        gl.BindTexture(gl::TEXTURE_2D, tex);

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

        gl.ActiveTexture(gl::TEXTURE0);

        let loc = gl.GetUniformLocation(shader_program.id(), c_str!("myTextureSampler").as_ptr());
        gl.Uniform1i(loc, 0);

        gl_int_p_matrix = gl.GetUniformLocation(shader_program.id(), c_str!("uPMatrix").as_ptr());
        gl.UniformMatrix4fv(gl_int_p_matrix, 1, gl::FALSE, projection_matrix.as_ptr());

        gl_int_mv_matrix = gl.GetUniformLocation(shader_program.id(), c_str!("uMVMatrix").as_ptr());
        gl.UniformMatrix4fv(gl_int_mv_matrix, 1, gl::FALSE, model_view_matrix.as_ptr());
    }

    // --------------------------------------------------------------------------------

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        use sdl2::event::{Event, WindowEvent};
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            }

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => unsafe {
                    gl.Viewport(0, 0, w as i32, h as i32);
                },
                _ => {}
            }
        }

        let ui = imgui_sdl2.frame(&window, &mut imgui, &event_pump.mouse_state());
        ui.show_demo_window(&mut true);

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        unsafe {
            gl.BindVertexArray(vao);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        }

        for rp in &seni_context.geometry.render_packets {
            unsafe {
                gl.BufferData(
                    gl::ARRAY_BUFFER,                                                       // target
                    (rp.geo.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                    rp.geo.as_ptr() as *const gl::types::GLvoid, // pointer to data
                    gl::STATIC_DRAW,                               // usage
                );

                gl.DrawArrays(
                    gl::TRIANGLE_STRIP, // mode
                    0,             // starting index in the enabled arrays
                    seni_context.geometry.render_packets[0].geo.len() as i32, // number of indices to be rendered
                );
            }
        }

        renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}


fn load_texture(ppath: &Path, name: &str) -> Result<BitmapInfo> {
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

fn identity() -> [f32; 16] {
    let out: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    out
}

fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [f32; 16] {
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

fn run_script(script: &str, settings: &config::Config) -> Result<Context> {
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
