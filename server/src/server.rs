use crate::error::Result;
use crate::staticfile::StaticFile;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
// use sqlx::PgPool;

pub use http::StatusCode;
pub use tide::{Request, Response, Server};

pub type Index = usize;
pub type PoorMansDb = BTreeMap<Index, DbRecord>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbRecord {
    pub id: Index,
    pub name: String,
    pub script: String,
}

pub struct ServerState {
    pub app_name: String,
    pub address: String,
    pub cookie_key: String,
    pub cookie_iv: String,
    pub cookie_secure: bool,
    pub session_path: String,
    pub static_file: StaticFile,
    pub poor_db: PoorMansDb,
}

pub fn ok_json(serializable: &impl Serialize) -> Result<Response> {
    Ok(Response::new(StatusCode::OK.as_u16()).body_json(serializable)?)
}

pub fn ok_string(s: String) -> Result<Response> {
    Ok(Response::new(StatusCode::OK.as_u16()).body_string(s))
}
