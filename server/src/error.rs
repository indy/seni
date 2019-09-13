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

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::error;
use std::fmt;
use std::io::Error as IoError;

pub type Result<T> = ::std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Serde(serde_json::error::Error),
    Request,
    ParseIntError(std::num::ParseIntError),
}
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(_) => write!(f, "IO error"),
            Error::Serde(_) => write!(f, "serde error"),
            Error::Request => write!(f, "request error"),
            Error::ParseIntError(_) => write!(f, "ParseIntError"),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseIntError(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error::Serde(e)
    }
}

impl ResponseError for Error {
    // builds the actual response to send back when an error occurs
    fn render_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .content_type("text/html; charset=utf-8")
            .body("error")
    }
}
