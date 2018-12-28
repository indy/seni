use clap::{App, Arg, ArgMatches, value_t};
use sen_core::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum NativeError {
    GeneralError,
    NoneError,
    IoError,
    SenError,
}

impl From<IoError> for NativeError {
    fn from(_e: IoError) -> NativeError {
        NativeError::IoError
    }
}

impl From<sen_core::Error> for NativeError {
    fn from(_e: sen_core::Error) -> NativeError {
        NativeError::SenError
    }
}

pub type Result<T> = ::std::result::Result<T, NativeError>;

fn main() {
    let matches = App::new("sen-native")
        .version("0.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("native cli build of seni")
        .arg(Arg::with_name("SCRIPT")
             .help("Sets the input seni script to use")
             .required(true)
             .index(1))
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .help("The seed to use")
                .takes_value(true),
        ).arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("debug mode")
                .takes_value(false),
        ).get_matches();

    if let Err(e) = run(&matches) {
        println!("Application error: {:?}", e);
    }
}

fn run(matches: &ArgMatches) -> Result<()> {
    if let Some(script) = matches.value_of("SCRIPT") {
        // this should always pass as SCRIPT is required
        println!("Using script file: {}", script);


        if matches.is_present("debug") {
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

pub fn read_script_file(filename: &str) -> Result<String> {
    let mut f = File::open(filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn run_debug_script(script: &str) -> Result<()> {
    let source = read_script_file(script)?;

    let program = compile_str(&source)?;
    println!("{}", &source);
    println!("{}", &program);

    Ok(())
}

fn run_debug_script_with_seed(_script: &str, _seed: u32) -> Result<()> {
    Ok(())
}

fn run_script(script: &str) -> Result<()> {
    let source = read_script_file(script)?;
    let res = compile_and_execute(&source)?;
    println!("res = {}", res);
    Ok(())
}

fn run_script_with_seed(_script: &str, _seed: u32) -> Result<()> {
    Ok(())
}
