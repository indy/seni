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

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder};

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::error::Error),
    Io(std::io::Error),
    NotInPoorDb(u32),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        match self {
            _ => Err(Status::InternalServerError),
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Error {
        Error::Serde(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

// don't need to implement any of the trait's methods
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Serde(_) => write!(f, "serde error"),
            Error::Io(_) => write!(f, "io error"),
            Error::NotInPoorDb(id) => write!(f, "id: {} not in PoorDb", id),
        }
    }
}
