use std::fs;
use std::path::Path;
use sass_rs::{Options, compile_file};

type Result<T> = ::std::result::Result<T, Box<dyn ::std::error::Error>>;

pub fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args.next();        // name of binary
    let path_to_scss_file = args.next().expect("Please pass in path to scss file");
    let path_to_output_file = args.next().expect("Please pass in path to the output file");

    let scss_file = Path::new(&path_to_scss_file);
    let out_file = Path::new(&path_to_output_file);

    let css = compile_file(&scss_file, Options::default())?;
    fs::write(out_file, css)?;

    Ok(())
}
