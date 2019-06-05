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

use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clap::{value_t, App, Arg};
use config;
use env_logger;
use gl;
use imgui;
use imgui::im_str;
use log::info;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
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
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .help("watch the scripts directory")
                .takes_value(false),
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

    if matches.is_present("watch") {
        run_watch(&config)
    } else {
        run(&config)
    }
}

fn run_watch(config: &config::Config) -> Result<()> {
    let assets_location = config.get_str("assets_path")?;
    let assets_path = Path::new(&assets_location);

    let bitmaps_location = config.get_str("bitmaps_path")?;
    let bitmaps_path = Path::new(&bitmaps_location);

    let script_filename = config.get_str("script")?;
    let scripts_path = config.get_str("scripts_path")?;
    let script_pathbuf = Path::new(&scripts_path).join(script_filename);

    let seni_source = seni::load_script(&script_pathbuf)?;
    let seni_context = seni::run_source(&seni_source, &config)?;

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
    let mut seni_renderer =
        render_seni::Renderer::new(&gl, &assets_path, &bitmaps_path, &seni_context)?;

    gl_util::update_viewport(&gl, viewport_width, viewport_height);

    // allow `num` to be shared across threads (Arc) and modified
    // (Mutex) safely without a data race.
    let num: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

    // create a cloned reference before moving `num` into the thread.
    let num_clone = num.clone();

    thread::spawn(move || {
        // some work here
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Create a watcher object, delivering debounced events.
        // The notification back-end is selected based on the platform.
        let mut watcher = watcher(tx, Duration::from_millis(100)).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.

        watcher
            .watch(scripts_path, RecursiveMode::Recursive)
            .unwrap();

        // receive debounced event
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::NoticeWrite(pathbuf)) => {
                    println!("noticed write: {:?}", pathbuf);
                    *num.lock().unwrap() = Some(pathbuf);
                }
                // Ok(DebouncedEvent::Write(pathbuf)) => {
                //     println!("write: {:?}", pathbuf);
                //     *num.lock().unwrap() = Some(pathbuf);
                // },
                Ok(event) => println!("other {:#?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    let mut event_pump = sdl_context.event_pump()?;

    // receive debounced event
    loop {
        // this is a bad place for this code
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
                } => return Ok(()),
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport_width = w as _;
                    viewport_height = h as _;
                }
                _ => {}
            }
        }

        gl_util::update_viewport(&gl, viewport_width, viewport_height);

        let ui = input_imgui.frame(&window, &mut imgui, &event_pump.mouse_state());
        // ui.show_demo_window(&mut true);

        // ~/repos/rust/imgui-rs/imgui-glium-examples/examples/test_window_impl.rs
        // ui.window(im_str!("Seni"))
        //     .position((0.0, 0.0), imgui::ImGuiCond::FirstUseEver)
        //     .size((800.0, 600.0), imgui::ImGuiCond::FirstUseEver)
        //     .build(|| {
        //         if ui.button(im_str!("Load.."), (0.0, 0.0)) {
        //             ui.open_popup(im_str!("modal"));
        //         }

        //         ui.popup_modal(im_str!("modal")).build(|| {
        //             ui.text("Content of my modal");
        //             if ui.button(im_str!("OK"), (0.0, 0.0)) {
        //                 ui.close_current_popup();
        //             }
        //         });

        //         ui.separator();
        //         if ui.collapsing_header(im_str!("script")).build() {
        //             ui.text(im_str!("{}", seni_source));
        //         }
        //     });

        let mutex_guard = num_clone.lock().unwrap().clone();
        if let Some(ref pathbuf) = mutex_guard {
            let seni_source = seni::load_script(&pathbuf)?;
            let seni_context = seni::run_source(&seni_source, &config)?;
            seni_renderer.rebake(&gl, &assets_path, &seni_context)?;

            println!("some pathbuf: {:?}", pathbuf);
            *num_clone.lock().unwrap() = None;
        }

        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        seni_renderer.render(viewport_width, viewport_height);
        imgui_renderer.render(ui);

        window.gl_swap_window();

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn run(config: &config::Config) -> Result<()> {
    let assets_location = config.get_str("assets_path")?;
    let assets_path = Path::new(&assets_location);

    let bitmaps_location = config.get_str("bitmaps_path")?;
    let bitmaps_path = Path::new(&bitmaps_location);

    let script_filename = config.get_str("script")?;
    let scripts_path = config.get_str("scripts_path")?;
    let script_pathbuf = Path::new(&scripts_path).join(script_filename);

    let seni_source = seni::load_script(&script_pathbuf)?;
    let seni_context = seni::run_source(&seni_source, &config)?;

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
        // ui.show_demo_window(&mut true);

        // ~/repos/rust/imgui-rs/imgui-glium-examples/examples/test_window_impl.rs
        ui.window(im_str!("Seni"))
            .position((0.0, 0.0), imgui::ImGuiCond::FirstUseEver)
            .size((800.0, 600.0), imgui::ImGuiCond::FirstUseEver)
            .build(|| {
                if ui.button(im_str!("Load.."), (0.0, 0.0)) {
                    ui.open_popup(im_str!("modal"));
                }

                ui.popup_modal(im_str!("modal")).build(|| {
                    ui.text("Content of my modal");
                    if ui.button(im_str!("OK"), (0.0, 0.0)) {
                        ui.close_current_popup();
                    }
                });

                ui.separator();
                if ui.collapsing_header(im_str!("script")).build() {
                    ui.text(im_str!("{}", seni_source));
                }
            });

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
