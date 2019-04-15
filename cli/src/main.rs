use clap::{value_t, App, Arg, ArgMatches};
use core::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error as IoError;

use log::{info, trace, error};
use env_logger;

#[derive(Debug)]
pub enum NativeError {
    GeneralError,
    NoneError,
    IoError(IoError),
    SenError(core::error::Error),
}

impl From<IoError> for NativeError {
    fn from(e: IoError) -> NativeError {
        NativeError::IoError(e)
    }
}

impl From<core::error::Error> for NativeError {
    fn from(e: core::error::Error) -> NativeError {
        NativeError::SenError(e)
    }
}

pub type Result<T> = ::std::result::Result<T, NativeError>;

fn main() {
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
                .help("debug mode")
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
    info!("env_logger initialised");

    if let Err(e) = run(&matches) {
        error!("Application error: {:?}", e);
    }
}

fn run(matches: &ArgMatches) -> Result<()> {
    trace!("run");

    if let Some(script) = matches.value_of("SCRIPT") {
        // this should always pass as SCRIPT is required
        info!("Using script file: {}", script);

        if matches.is_present("packed_trait_list") {
            print_packed_trait_list(script)?;
        } else if matches.is_present("debug") {
            if let Ok(seed) = value_t!(matches.value_of("seed"), u32) {
                run_debug_script_with_seed(script, seed)?;
            } else {
                run_debug_script(script)?;
            }
        } else {
            if let Ok(seed) = value_t!(matches.value_of("seed"), u32) {
                run_script_with_seed(script, seed)?;
            } else {
                run_script(script)?;
            }
        }
    }

    Ok(())
}

fn read_script_file(filename: &str) -> Result<String> {
    trace!("read_script_file");

    let mut f = File::open(filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    Ok(contents)
}

fn run_debug_script(script: &str) -> Result<()> {
    trace!("run_debug_script");

    let source = read_script_file(script)?;
    let program = compile_str(&source)?;

    println!("{}", &source);
    println!("{}", &program);

    Ok(())
}

fn run_debug_script_with_seed(_script: &str, _seed: u32) -> Result<()> {
    trace!("run_debug_script_with_seed");

    Ok(())
}

fn run_script(script: &str) -> Result<()> {
    trace!("run_script");

    let source = read_script_file(script)?;
    let res = compile_and_execute(&source)?;

    println!("res = {}", res);

    Ok(())
}

fn run_script_with_seed(_script: &str, _seed: u32) -> Result<()> {
    trace!("run_script_with_seed");

    Ok(())
}

fn print_packed_trait_list(script: &str) -> Result<()> {
    trace!("print_packed_trait_list");

    let source = read_script_file(script)?;
    let trait_list = build_traits(&source)?;
    let mut packed: String = "".to_string();

    trait_list.pack(&mut packed)?;
    println!("{}", packed);

    Ok(())
}
