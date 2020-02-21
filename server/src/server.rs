use crate::error::Result;
use crate::staticfile::StaticFile;
use serde::Serialize;
use sqlx::PgPool;

pub use http::StatusCode;
pub use tide::{Request, Response, Server};

pub struct ServerState {
    pub app_name: String,
    pub address: String,
    pub cookie_key: String,
    pub cookie_iv: String,
    pub cookie_secure: bool,
    pub session_path: String,
    pub static_file: StaticFile,
    pub pool: PgPool,
}

pub fn ok_json(serializable: &impl Serialize) -> Result<Response> {
    Ok(Response::new(StatusCode::OK.as_u16()).body_json(serializable)?)
}

pub fn ok_string(s: String) -> Result<Response> {
    Ok(Response::new(StatusCode::OK.as_u16()).body_string(s))
}
