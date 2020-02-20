use crate::error::{Error, Result};
use crate::server::{ok_json, ok_string, DbRecord, Request, Response, ServerState};

pub async fn gallery_item(req: Request<ServerState>) -> Result<Response> {
    // get parameters
    //
    let gallery_id: usize = req.param("id")?;

    // db statement
    //
    let state = req.state();
    let db = &state.poor_db;
    match db.get(&gallery_id) {
        Some(sketch) => ok_string(sketch.script.to_string()),
        None => Err(Error::Authenticating), // todo: this is the wrong error code, fix later
    }
}

pub async fn gallery(req: Request<ServerState>) -> Result<Response> {
    // db statement
    //
    let state = req.state();
    let db = &state.poor_db;
    let gallery: Vec<DbRecord> = db.values().rev().cloned().collect();

    ok_json(&gallery)
}
