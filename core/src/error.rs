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

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    // these are lazy errors, used during dev as basically placeholder errors. remove them
    GeneralError,
    NotedError(String),

    // conversions from other errors
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseStrumError(strum::ParseError),

    // colour
    //
    IncorrectColourFormat,
    InvalidColourHue,
    InvalidColourChannel,
    Colour(String),

    // parser
    ParserInvalidChar(char),
    ParserInvalidLiteral,
    ParserUnableToParseFloat(String),
    ParserHandledToken(String),

    // mem
    MemUnmappableBytecodeArg,
    MemUnmappableI32,

    // compiler
    CompilerFnWithoutName,
    CompilerFnDeclIncomplete,
    Compiler(String),

    // vm
    VMStackUnderflow,
    VMStackOverflow,
    VM(String),

    // bind
    Bind(String),

    // native
    Native(String),

    // interp
    Interp(String),

    // path
    Path(String),

    // geometry
    Geometry(String),

    // gene
    Gene(String),

    // packable
    Packable(String),

    Unparser(String),
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(e: std::num::ParseFloatError) -> Error {
        Error::ParseFloatError(e)
    }
}

impl From<strum::ParseError> for Error {
    fn from(e: strum::ParseError) -> Error {
        Error::ParseStrumError(e)
    }
}

// don't need to implement any of the trait's methods
impl error::Error for Error {
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::GeneralError => write!(f, "seni core: General Error"),
            Error::NotedError(s) => write!(f, "seni core: Noted Error: {}", s),
            Error::ParseIntError(_) => write!(f, "seni core: ParseIntError"),
            Error::ParseFloatError(_) => write!(f, "seni core: ParseFloatError"),
            Error::ParseStrumError(_) => write!(f, "seni core: ParseStrumError"),
            Error::IncorrectColourFormat => write!(f, "seni core: IncorrectColourFormat"),
            Error::InvalidColourHue => write!(f, "seni core: InvalidColourHue"),
            Error::InvalidColourChannel => write!(f, "seni core: InvalidColourChannel"),
            Error::Colour(s) => write!(f, "seni core: Colour {}", s),
            Error::ParserInvalidChar(c) => write!(f, "seni core: ParserInvalidChar: {}", c),
            Error::ParserInvalidLiteral => write!(f, "seni core: ParserInvalidLiteral"),
            Error::ParserUnableToParseFloat(s) => write!(f, "seni core: ParserUnableToParseFloat {}", s),
            Error::ParserHandledToken(s) => write!(f, "seni core: ParserHandledToken {}", s),
            Error::MemUnmappableBytecodeArg => write!(f, "seni core: MemUnmappableBytecodeArg"),
            Error::MemUnmappableI32 => write!(f, "seni core: MemUnmappableI32"),
            Error::CompilerFnWithoutName => write!(f, "seni core: CompilerFnWithoutName"),
            Error::CompilerFnDeclIncomplete => write!(f, "seni core: CompilerFnDeclIncomplete"),
            Error::Compiler(s) => write!(f, "seni core: Compiler {}", s),
            Error::VMStackUnderflow => write!(f, "seni core: VMStackUnderflow"),
            Error::VMStackOverflow => write!(f, "seni core: VMStackOverflow"),
            Error::VM(s) => write!(f, "seni core: VM: {}", s),
            Error::Bind(s) => write!(f, "seni core: Bind: {}", s),
            Error::Native(s) => write!(f, "seni core: Native: {}", s),
            Error::Interp(s) => write!(f, "seni core: Interp: {}", s),
            Error::Path(s) => write!(f, "seni core: Path: {}", s),
            Error::Geometry(s) => write!(f, "seni core: Geometry: {}", s),
            Error::Gene(s) => write!(f, "seni core: Gene: {}", s),
            Error::Packable(s) => write!(f, "seni core: Packable: {}", s),
            Error::Unparser(s) => write!(f, "seni core: Unparser: {}", s),
        }
    }
}
