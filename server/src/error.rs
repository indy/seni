use crate::server::{self, Response, StatusCode};
use std::error;
use std::fmt;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Var(std::env::VarError),
    Sqlx(sqlx::Error),
    Argon2(argon2::Error),
    Serde(serde_json::Error),
    Tide(tide::Error),
    Base64Decode(base64::DecodeError),
    BlockMode(block_modes::BlockModeError),
    KeyLength(block_modes::InvalidKeyIvLength),
    Utf8(std::str::Utf8Error),
    ParseInt(std::num::ParseIntError),
    Http(server::StatusCode),
    Authenticating,
    Other,
}

impl From<Error> for Response {
    fn from(e: Error) -> Response {
        let status_code = match e {
            Error::Authenticating => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        server::Response::new(status_code.as_u16())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Error {
        Error::Var(e)
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Error {
        Error::Sqlx(e)
    }
}

impl From<argon2::Error> for Error {
    fn from(e: argon2::Error) -> Error {
        Error::Argon2(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::Serde(e)
    }
}

impl From<tide::Error> for Error {
    fn from(e: tide::Error) -> Error {
        Error::Tide(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Error {
        Error::Base64Decode(e)
    }
}

impl From<block_modes::BlockModeError> for Error {
    fn from(e: block_modes::BlockModeError) -> Error {
        Error::BlockMode(e)
    }
}

impl From<block_modes::InvalidKeyIvLength> for Error {
    fn from(e: block_modes::InvalidKeyIvLength) -> Error {
        Error::KeyLength(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Error {
        Error::Utf8(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Error {
        Error::ParseInt(e)
    }
}

// don't need to implement any of the trait's methods
impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(_) => write!(f, "tidal: IO Error"),
            Error::Var(_) => write!(f, "tidal: Var Error"),
            Error::Sqlx(_) => write!(f, "tidal: Sqlx Error"),
            Error::Argon2(_) => write!(f, "tidal: Argon2 Error"),
            Error::Serde(_) => write!(f, "tidal: Serde Error"),
            Error::Tide(_) => write!(f, "tidal: Tide Error"),
            Error::Base64Decode(_) => write!(f, "tidal: Base64Decode Error"),
            Error::BlockMode(_) => write!(f, "tidal: BlockMode Error"),
            Error::KeyLength(_) => write!(f, "tidal: KeyLength Error"),
            Error::Utf8(_) => write!(f, "tidal: Utf8Error Error"),
            Error::ParseInt(_) => write!(f, "tidal: ParseInt Error"),
            Error::Http(code) => write!(f, "tidal: status code {}", code),
            Error::Authenticating => write!(f, "tidal: authenticating"),
            Error::Other => write!(f, "tidal: some other error"),
        }
    }
}
