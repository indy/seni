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
mod resources;

use clap::{value_t, App, Arg};
use config;
use env_logger;
use image::GenericImageView;
use log::info;

use gl;
use imgui;
use imgui_opengl_renderer;
use imgui_sdl2;
use sdl2;

use crate::error::Result;
use crate::resources::Resources;
use std::path::Path;

use core::BitmapInfo;

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

fn load_texture(res: &Resources, name: &str) -> Result<BitmapInfo> {
    let path = res.resource_path(name)?;

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

    let bitmap_info = BitmapInfo {
        width,
        height,
        data,
        ..Default::default()
    };

    Ok(bitmap_info)
}

fn run(_config: &config::Config) -> Result<()> {
    let res = Resources::from_relative_exe_path(Path::new("assets"))?;

    let _bitmap_info = load_texture(&res, "textures/texture.png")?;

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
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/triangle")?;

    // set up vertex buffer object
    //
    let vertices: Vec<f32> = vec![
        // pos      // colour           // uv
        // x,  y,   r,   g,   b,   a,   u,   v
        0.5, -0.5, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, // bottom left
        0.0, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, // top
        1.0, -0.75, 1.0, 0.0, 0.0, 0.1, 0.0, 0.0, // bottom right
        0.0, -0.75, 0.0, 1.0, 0.0, 0.1, 1.0, 0.0, // bottom left
        0.5, 0.25, 0.0, 0.0, 1.0, 0.1, 1.0, 1.0, // top
    ];

    let mut vbo: gl::types::GLuint = 0;
    let mut vao: gl::types::GLuint = 0;

    unsafe {
        // set up vertex buffer object
        //
        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,                                                       // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
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
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);

        // assuming that we'll be using pre-multiplied alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        gl.Enable(gl::BLEND);
        gl.BlendEquation(gl::FUNC_ADD);
        gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
    }

    // --------------------------------------------------------------------------------

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        use sdl2::event::Event;
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
                _ => {}
            }
        }

        let ui = imgui_sdl2.frame(&window, &mut imgui, &event_pump.mouse_state());
        ui.show_demo_window(&mut true);

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // draw triangle

        shader_program.set_used();
        unsafe {
            gl.BindVertexArray(vao);
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                6,             // number of indices to be rendered
            );
        }

        renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
