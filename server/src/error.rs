// Copyright (C) 2020 Inderjit Gill <email@indy.io>

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

use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, From};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Display, From, Debug)]
pub enum Error {
    NotFound,
    TokioPostgres(tokio_postgres::error::Error),
    TokioPostgresMapper(tokio_pg_mapper::Error),
    DeadPool(deadpool_postgres::PoolError),
    DeadPoolConfig(deadpool_postgres::config::ConfigError),
    Actix(actix_web::Error),
    IO(std::io::Error),
    Var(std::env::VarError),
    Argon2(argon2::Error),
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    Authenticating,
    Other,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Error::NotFound => HttpResponse::NotFound().finish(),
            Error::DeadPool(ref err) => HttpResponse::InternalServerError().body(err.to_string()),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
