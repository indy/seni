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
    ConfigError(config::ConfigError),
    StringError(String),
    SDL2WindowBuildError(sdl2::video::WindowBuildError),
}

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

// don't need to implement any of the trait's methods
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ConfigError(c) => write!(f, "seni gui: Config Error: {:?}", c),
            Error::StringError(s) => write!(f, "seni gui: String Error: {}", s),
            Error::SDL2WindowBuildError(e) => write!(f, "seni gui: SDL2WindowBuildError: {:?}", e),
        }
    }
}
