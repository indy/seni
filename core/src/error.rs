// Copyright (C) 2019 Inderjit Gill

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::error;
use std::fmt;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Bitmap,
    BitmapCache,
    Colour,
    Compiler,
    Context,
    Gene,
    Geometry,
    Lexer,
    Native,
    Node,
    Packable,
    Parser,
    Program,
    Unparser,
    VM,
    // conversions from other errors
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseStrumError(strum::ParseError),
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
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Bitmap => write!(f, "seni core: Bitmap"),
            Error::BitmapCache => write!(f, "seni core: BitmapCache"),
            Error::Colour => write!(f, "seni core: Colour"),
            Error::Compiler => write!(f, "seni core: Compiler"),
            Error::Context => write!(f, "seni core: Context"),
            Error::Gene => write!(f, "seni core: Gene"),
            Error::Geometry => write!(f, "seni core: Geometry"),
            Error::Lexer => write!(f, "seni core: Lexer"),
            Error::Native => write!(f, "seni core: Native"),
            Error::Node => write!(f, "seni core: Node"),
            Error::Packable => write!(f, "seni core: Packable"),
            Error::Parser => write!(f, "seni core: Parser"),
            Error::Program => write!(f, "seni core: Program"),
            Error::Unparser => write!(f, "seni core: Unparser"),
            Error::VM => write!(f, "seni core: VM"),
            // conversions from other errors
            Error::ParseIntError(_) => write!(f, "seni core: ParseIntError"),
            Error::ParseFloatError(_) => write!(f, "seni core: ParseFloatError"),
            Error::ParseStrumError(_) => write!(f, "seni core: ParseStrumError"),
        }
    }
}
