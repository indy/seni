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

/// read in a json file from https://github.com/Jam3/nice-color-palettes.git
/// and output a rust file which lists the palettes in hsluv colour space

use clap::{App, Arg};
use core::colour::{Colour, ColourFormat};
use serde_json;

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn main() -> Result<()> {
    // update config with command line options
    //
    let matches = App::new("palette-maker")
        .version("4.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("generates rust source code listing colour palettes")
        .arg(
            Arg::with_name("SOURCE")
                .help("Sets the json file to use")
                .index(1),
        )
        .arg(
            Arg::with_name("DEST")
                .help("The output rust source filename")
                .index(2),
        )
        .get_matches();

    if let Some(json_filename) = matches.value_of("SOURCE") {
        if let Some(rs_filename) = matches.value_of("DEST") {
            run(json_filename, rs_filename)?;
        }
    }

    Ok(())
}

fn run(json_filename: &str, rs_filename: &str) -> Result<()> {
    let contents = std::fs::read_to_string(json_filename)?;
    let json_palettes: Vec<Vec<String>> = serde_json::from_str(&contents)?;
    let converted_palettes: Vec<Vec<Colour>> = convert_palettes(json_palettes)?;
    let source_code = build_source(&converted_palettes)?;

    std::fs::write(rs_filename, source_code)?;

    Ok(())
}

fn build_source(palettes: &Vec<Vec<Colour>>) -> Result<String> {
    let source_size = estimate_output_size(palettes.len());
    let mut source = String::with_capacity(source_size);

    source.push_str(get_header());

    for palette in palettes {
        source.push_str("    &[\n");
        for colour in palette {
            source.push_str("        Colour {\n");
            source.push_str("            format: ColourFormat::Hsluv,\n");
            source.push_str(&format!("            e0: {},\n", colour.e0));
            source.push_str(&format!("            e1: {},\n", colour.e1));
            source.push_str(&format!("            e2: {},\n", colour.e2));
            source.push_str("            e3: 1.0,\n");
            source.push_str("        },\n")
        }
        source.push_str("    ],\n");
    }

    source.push_str(get_footer());

    Ok(source)
}

fn get_header() -> &'static str {
    "// Copyright (C) 2019 Inderjit Gill

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


use crate::colour::{Colour, ColourFormat};

pub const COLOUR_PALETTES2: &[&[Colour]] = &[
"
}

fn get_footer() -> &'static str {
    "];
"
}

fn estimate_output_size(num_palettes: usize) -> usize {
    const CHARACTERS_PER_LINE: usize = 420;
    const HEAD_OF_FILE: usize = 20 * 70;

    (num_palettes * CHARACTERS_PER_LINE) + HEAD_OF_FILE
}

fn convert_palettes(palettes: Vec<Vec<String>>) -> Result<Vec<Vec<Colour>>> {
    palettes
        .into_iter()
        .map(|palette| convert_palette(palette))
        .collect()
}

fn convert_palette(palette: Vec<String>) -> Result<Vec<Colour>> {
    palette
        .into_iter()
        .map(|colour| convert_colour(colour))
        .collect()
}

fn convert_colour(hex: String) -> Result<Colour> {
    let rgb = Colour::new(
        ColourFormat::Rgb,
        normalised_colour_from_hex_string(&hex[1..3])?,
        normalised_colour_from_hex_string(&hex[3..5])?,
        normalised_colour_from_hex_string(&hex[5..])?,
        1.0,
    );

    let hsluv = rgb.convert(ColourFormat::Hsluv)?;

    Ok(hsluv)
}

fn normalised_colour_from_hex_string(hex_component: &str) -> Result<f32> {
    let value = i32::from_str_radix(hex_component, 16)?;
    Ok(value as f32 / 255.0)
}
