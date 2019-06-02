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

// Order of mod matters!. Declare gl_util before using it in other modules
#[macro_use]
mod gl_util;
mod matrix_util;

mod error;
mod input_imgui;
mod render_gl;
mod render_imgui;
mod render_piece;
mod render_seni;
mod render_square;
mod seni;

use std::path::Path;

use clap::{value_t, App, Arg};
use config;
use env_logger;
use gl;
use imgui;
use log::info;
use sdl2;

use crate::error::Result;

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

    let bitmaps_location = config.get_str("bitmaps_path")?;
    let bitmaps_path = Path::new(&bitmaps_location);

    let script_filename = config.get_str("script")?;

    let seni_context = seni::run_script(&script_filename, &config)?;

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

    let mut input_imgui = input_imgui::ImguiSdl2::new(&mut imgui);

    let mut viewport_width: usize = 900;
    let mut viewport_height: usize = 700;

    unsafe {
        // set up shared state for window
        //
        gl.ClearColor(1.0, 1.0, 1.0, 1.0);

        // assuming that we'll be using pre-multiplied alpha
        // see http://www.realtimerendering.com/blog/gpus-prefer-premultiplication/
        gl.Enable(gl::BLEND);
        gl.BlendEquation(gl::FUNC_ADD);
        gl.BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
    }

    let imgui_renderer = render_imgui::Renderer::new(&gl, &assets_path, &mut imgui)?;
    let seni_renderer =
        render_seni::Renderer::new(&gl, &assets_path, &bitmaps_path, &seni_context)?;

    gl_util::update_viewport(&gl, viewport_width, viewport_height);

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        use sdl2::event::{Event, WindowEvent};
        use sdl2::keyboard::Keycode;

        for event in event_pump.poll_iter() {
            input_imgui.handle_event(&mut imgui, &event);
            if input_imgui.ignore_event(&event) {
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
                } => {
                    viewport_width = w as _;
                    viewport_height = h as _;
                    gl_util::update_viewport(&gl, viewport_width, viewport_height);
                }
                _ => {}
            }
        }

        let ui = input_imgui.frame(&window, &mut imgui, &event_pump.mouse_state());
        ui.show_demo_window(&mut true);

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        seni_renderer.render(viewport_width, viewport_height);
        imgui_renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
