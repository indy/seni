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

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Placeholder(String),        // temp placeholder error

    ConfigError(config::ConfigError),
    StringError(String),
    SDL2WindowBuildError(sdl2::video::WindowBuildError),
    FFINulError(std::ffi::NulError),
    Io(std::io::Error),
    FileContainsNil,
    FailedToGetExePath,
    ResourceLoad {
        name: String,
//        inner: Error,
    },
    CanNotDetermineShaderTypeForResource {
        name: String,
    },
    CompileError {
        name: String,
        message: String,
    },
    LinkError {
        name: String,
        message: String,
    },}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Error {
        Error::ConfigError(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::StringError(e)
    }
}

impl From<sdl2::video::WindowBuildError> for Error {
    fn from(e: sdl2::video::WindowBuildError) -> Error {
        Error::SDL2WindowBuildError(e)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Error {
        Error::FFINulError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::Io(other)
    }
}

// don't need to implement any of the trait's methods
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Placeholder(s) => write!(f, "seni gui: Placeholder: {:?}", s),
            Error::ConfigError(c) => write!(f, "seni gui: Config Error: {:?}", c),
            Error::StringError(s) => write!(f, "seni gui: String Error: {}", s),
            Error::SDL2WindowBuildError(e) => write!(f, "seni gui: SDL2WindowBuildError: {:?}", e),
            Error::FFINulError(e) => write!(f, "seni gui: std::ffi:NulError: {:?}", e),
            Error::Io(e) => write!(f, "seni gui: Io: {:?}", e),
            Error::FileContainsNil => write!(f, "seni gui: FileContainsNil"),
            Error::FailedToGetExePath => write!(f, "seni gui: FailedToGetExePath"),
            Error::ResourceLoad{name} => write!(f, "seni gui: {}", name),
            Error::CanNotDetermineShaderTypeForResource{name} => write!(f, "seni gui: {}", name),
            Error::CompileError{name, message} => write!(f, "seni gui: {} {}", name, message),
            Error::LinkError{name, message} => write!(f, "seni gui: {} {}", name, message),
        }
    }
}
