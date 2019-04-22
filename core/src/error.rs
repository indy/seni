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

pub type Result<T> = ::std::result::Result<T, Error>;

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
